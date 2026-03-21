"""
Analyzer - NVIDIA LLM Deep Analysis

Refactored from batch_llm_analyzer.py.

Takes high-severity findings from DB, sends to NVIDIA endpoint for deep analysis.
Caches results in llm_analyses table. Uses NVIDIA_API_KEY from env.

Note: Cerebras triage is handled by the Rust CLI (--llm flag).
This module is ONLY for NVIDIA deep analysis of flagged findings.

Usage:
    from harness.core import Analyzer, Store
    
    store = Store("data/corpus.db")
    analyzer = Analyzer(store)
    
    # Get packages needing LLM analysis
    packages = store.get_flagged_for_llm(min_severity="high")
    
    # Analyze with NVIDIA
    for pkg in packages:
        result = analyzer.analyze_package(pkg["name"], pkg["version"])
        store.save_llm_analysis(pkg["name"], pkg["version"], pkg["tarball_sha256"], result)
"""

import json
import os
import tarfile
import tempfile
from pathlib import Path
from typing import Optional, Dict, Any, List
import requests

# Configuration - read from environment with fallbacks
NVIDIA_API_KEY = os.environ.get("NVIDIA_API_KEY", "")
NVIDIA_BASE_URL = os.environ.get(
    "NVIDIA_BASE_URL",
    "https://integrate.api.nvidia.com/v1",
)

# Model list in order of preference (strongest first)
# Read from environment or use defaults
DEFAULT_MODELS = [
    "qwen/qwen3.5-397b-a17b",  # Strongest - Qwen 3.5 397B
    "moonshotai/kimi-k2.5",     # Kimi K2.5
    "z-ai/glm5",                # GLM-5
    "meta/llama3-70b-instruct", # Fallback - Llama 3 70B
]

NVIDIA_MODELS = os.environ.get(
    "NVIDIA_MODELS",
    ",".join(DEFAULT_MODELS)
).split(",")

# LLM prompt for malware analysis
LLM_PROMPT = """You are a security analyst reviewing a npm package for potential malware.

The package has been flagged by static analysis with the following findings:
{findings}

Review the package structure and code. Determine:
1. Is this package malicious? (yes/no/uncertain)
2. What is the confidence level? (high/medium/low)
3. What specific behaviors are concerning?
4. What is your recommendation? (quarantine/monitor/clean)

Respond in JSON format:
{{
    "malicious": "yes|no|uncertain",
    "confidence": "high|medium|low",
    "concerns": ["list", "of", "concerns"],
    "recommendation": "quarantine|monitor|clean",
    "reasoning": "brief explanation"
}}
"""


class Analyzer:
    """
    NVIDIA LLM analyzer for deep analysis of flagged packages.
    
    Refactored from batch_llm_analyzer.py.
    Only handles NVIDIA deep analysis - Cerebras triage is in Rust CLI.
    
    Uses model fallback: tries models in order until one succeeds.
    """

    def __init__(self, store: Optional[Any] = None):
        """
        Initialize analyzer.
        
        Args:
            store: Optional Store instance for caching
        """
        self.store = store
        self.api_key = NVIDIA_API_KEY
        self.base_url = NVIDIA_BASE_URL
        self.models = NVIDIA_MODELS  # List of models to try in order

    def _call_nvidia_with_fallback(self, prompt: str) -> Optional[Dict]:
        """
        Call NVIDIA API with model fallback.
        
        Tries models in order until one succeeds.
        
        Args:
            prompt: Analysis prompt
            
        Returns:
            Parsed JSON response or None
        """
        if not self.api_key:
            return None
        
        last_error = None
        
        for model in self.models:
            try:
                result = self._call_nvidia_model(prompt, model)
                if result:
                    result["model_used"] = model
                    return result
            except Exception as e:
                last_error = e
                continue  # Try next model
        
        return None

    def _call_nvidia_model(self, prompt: str, model: str) -> Optional[Dict]:
        """
        Call NVIDIA API with specific model.
        
        Args:
            prompt: Analysis prompt
            model: Model name to use
            
        Returns:
            Parsed JSON response or None
        """
        resp = requests.post(
            f"{self.base_url}/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": model,
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a security analyst specializing in npm package malware detection.",
                    },
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.1,
                "max_tokens": 1000,
            },
            timeout=120,
        )
        
        if resp.status_code == 200:
            data = resp.json()
            content = data["choices"][0]["message"]["content"]
            return json.loads(content)
        elif resp.status_code == 400:
            # Model not available, try next
            return None
        else:
            resp.raise_for_status()
        
        return None

    def _extract_package(self, tarball_path: str) -> Optional[Path]:
        """Extract tarball and return package directory."""
        try:
            tmpdir = tempfile.mkdtemp()
            with tarfile.open(tarball_path, "r:gz") as tar:
                tar.extractall(tmpdir, filter="data")
            
            pkg_dir = Path(tmpdir) / "package"
            if not pkg_dir.exists():
                dirs = [d for d in Path(tmpdir).iterdir() if d.is_dir()]
                if dirs:
                    pkg_dir = dirs[0]
            
            return pkg_dir if pkg_dir.exists() else None
        except Exception:
            return None

    def _get_package_files(self, pkg_dir: Path, max_files: int = 50) -> Dict[str, str]:
        """
        Get package file contents for LLM analysis.
        
        Args:
            pkg_dir: Package directory
            max_files: Maximum files to include
            
        Returns:
            Dict mapping filepath to content
        """
        files = {}
        
        # Priority files
        priority = ["package.json", "install.js", "preinstall.js", "postinstall.js", "index.js"]
        
        for filename in priority:
            filepath = pkg_dir / filename
            if filepath.exists():
                try:
                    content = filepath.read_text(errors="ignore")
                    files[filename] = content[:10000]  # Limit size
                except Exception:
                    pass
        
        # Add more files if needed
        if len(files) < max_files:
            for filepath in pkg_dir.rglob("*.js"):
                if len(files) >= max_files:
                    break
                if filepath.name.startswith("."):
                    continue
                if "node_modules" in str(filepath):
                    continue
                try:
                    rel_path = str(filepath.relative_to(pkg_dir))
                    if rel_path not in files:
                        content = filepath.read_text(errors="ignore")
                        files[rel_path] = content[:5000]
                except Exception:
                    pass
        
        return files

    def analyze_package(
        self,
        name: str,
        version: str,
        tarball_path: Optional[str] = None,
        findings: Optional[List[Dict]] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        Analyze a package with NVIDIA LLM.
        
        Args:
            name: Package name
            version: Package version
            tarball_path: Path to package tarball
            findings: List of static analysis findings
            
        Returns:
            Analysis result dict or None
        """
        # Check cache
        if self.store and tarball_path:
            from .fetcher import Fetcher
            fetcher = Fetcher()
            dl_info = fetcher.download_package(f"{name}@{version}")
            if dl_info:
                cached = self.store.get_llm_analysis(
                    name, version, dl_info["tarball_sha256"]
                )
                if cached:
                    return cached
        
        # Extract package
        if not tarball_path:
            from .fetcher import Fetcher
            fetcher = Fetcher()
            dl_info = fetcher.download_package(f"{name}@{version}")
            if not dl_info:
                return None
            tarball_path = dl_info["tarball_path"]
        
        pkg_dir = self._extract_package(tarball_path)
        if not pkg_dir:
            return None
        
        # Get file contents
        files = self._get_package_files(pkg_dir)
        
        # Format findings for prompt
        findings_text = ""
        if findings:
            for f in findings[:20]:  # Limit findings
                findings_text += f"- {f.get('severity', '')}: {f.get('description', '')}\n"
        
        # Build prompt
        file_summary = "\n".join([
            f"=== {path} ===\n{content[:2000]}..."
            for path, content in list(files.items())[:10]
        ])
        
        prompt = LLM_PROMPT.format(findings=findings_text or "No static findings.")
        prompt += f"\n\nPackage files:\n{file_summary}"

        # Call NVIDIA with model fallback
        result = self._call_nvidia_with_fallback(prompt)

        if result:
            result["analyzed_at"] = __import__("datetime").datetime.utcnow().isoformat()
            result["package"] = name
            result["version"] = version
            # model_used is already set by _call_nvidia_with_fallback
            
            # Cache result
            if self.store and tarball_path:
                from .fetcher import Fetcher
                fetcher = Fetcher()
                dl_info = fetcher.download_package(f"{name}@{version}")
                if dl_info:
                    self.store.save_llm_analysis(
                        name, version, dl_info["tarball_sha256"], result
                    )
        
        # Cleanup
        try:
            import shutil
            shutil.rmtree(pkg_dir.parent)
        except Exception:
            pass
        
        return result

    def analyze_batch(
        self,
        packages: List[Dict],
        max_workers: int = 3,
    ) -> List[Dict]:
        """
        Analyze multiple packages in parallel.
        
        Args:
            packages: List of package dicts with name, version, tarball_path
            max_workers: Maximum parallel workers
            
        Returns:
            List of analysis results
        """
        from concurrent.futures import ThreadPoolExecutor, as_completed
        
        results = []
        
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            futures = {
                executor.submit(
                    self.analyze_package,
                    pkg.get("name", ""),
                    pkg.get("version", ""),
                    pkg.get("tarball_path"),
                    pkg.get("findings", []),
                ): pkg
                for pkg in packages
            }
            
            for future in as_completed(futures):
                pkg = futures[future]
                try:
                    result = future.result()
                    if result:
                        results.append(result)
                except Exception as e:
                    results.append({
                        "package": pkg.get("name", ""),
                        "version": pkg.get("version", ""),
                        "error": str(e),
                        "analyzed_at": __import__("datetime").datetime.utcnow().isoformat(),
                    })
        
        return results
