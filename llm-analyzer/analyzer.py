#!/usr/bin/env python3
"""
glassware LLM Analyzer - Serial Version

Analyzes glassware security findings using NVIDIA NIM API
Mimics manual analysis process for consistency
"""

import os
import json
import requests
from pathlib import Path
from datetime import datetime
from typing import Optional

# Configuration
NVIDIA_API_KEY = os.environ.get("NVIDIA_API_KEY", "")
NVIDIA_BASE_URL = "https://integrate.api.nvidia.com/v1"
MODEL = "meta/llama-3.3-70b-instruct"  # High-capability model, available on NVIDIA NIM

# Analysis prompt template (based on manual analysis pattern)
ANALYSIS_PROMPT = """You are a security analyst specializing in npm supply chain attacks, particularly GlassWare/GlassWorm Unicode steganography attacks.

Analyze this security finding from a scanned npm package.

## Package Information
- **Name:** {package_name}
- **Version:** {package_version}
- **Author:** {author}
- **Description:** {description}

## Security Finding
- **File:** {file_path}
- **Line:** {line}
- **Category:** {category}
- **Severity:** {severity}
- **Message:** {message}

## Source Code Context
```
{source_context}
```

## Analysis Tasks

1. **Author Assessment**: Is the author anonymous, known entity, or suspicious?

2. **Pattern Classification**: Is this detected pattern:
   - MALICIOUS (clear attack pattern, no legitimate use)
   - SUSPICIOUS (could be either, needs human review)
   - FALSE_POSITIVE (legitimate code, common pattern)

3. **Context Analysis**: Consider:
   - Is this a bundled/minified file?
   - Is this a documentation file?
   - Is this a legitimate crypto library?
   - Is this an MCP server / high-value target?

4. **Confidence Score**: 0.0 to 1.0 based on evidence strength

## Response Format

Respond with ONLY valid JSON (no markdown, no explanation outside JSON):

{{
  "classification": "MALICIOUS" | "SUSPICIOUS" | "FALSE_POSITIVE",
  "confidence": 0.0-1.0,
  "reasoning": "2-3 sentence explanation of your classification",
  "indicators": ["list", "of", "key", "technical", "indicators"],
  "recommended_action": "REPORT_IMMEDIATELY" | "INVESTIGATE_FURTHER" | "IGNORE",
  "notes": "Any additional observations"
}}
"""


class NIMAnalyzer:
    """NVIDIA NIM-based security finding analyzer"""
    
    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key or NVIDIA_API_KEY
        if not self.api_key:
            raise ValueError(
                "NVIDIA_API_KEY not set. "
                "Set GLASSWARE_NVIDIA_API_KEY env var or pass api_key to constructor."
            )
        
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        })
    
    def analyze_finding(
        self,
        finding: dict,
        package_info: dict,
        source_context: str = "",
        max_context_lines: int = 100,
    ) -> dict:
        """
        Analyze a single security finding
        
        Args:
            finding: glassware finding JSON
            package_info: Package metadata
            source_context: Source code around the finding
            max_context_lines: Max lines of context to include
        
        Returns:
            Analysis result JSON
        """
        # Build prompt
        prompt = ANALYSIS_PROMPT.format(
            package_name=package_info.get("name", "unknown"),
            package_version=package_info.get("version", "unknown"),
            author=package_info.get("author", "unknown"),
            description=package_info.get("description", "unknown"),
            file_path=finding.get("file", "unknown"),
            line=finding.get("line", 0),
            category=finding.get("category", "unknown"),
            severity=finding.get("severity", "unknown"),
            message=finding.get("message", ""),
            source_context=source_context[:50000],  # Truncate to avoid token limits
        )
        
        # Build API request
        payload = {
            "model": MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a security analyst specializing in npm supply chain attacks. Respond with ONLY valid JSON.",
                },
                {
                    "role": "user",
                    "content": prompt,
                },
            ],
            "temperature": 0.1,  # Low temperature for consistent analysis
            "max_tokens": 1024,
        }
        
        # Make API call
        try:
            response = self.session.post(
                f"{NVIDIA_BASE_URL}/chat/completions",
                json=payload,
                timeout=120,  # 2 minute timeout
            )
            response.raise_for_status()
            
            # Parse response
            result = response.json()
            content = result["choices"][0]["message"]["content"]
            
            # Extract JSON from response (strip markdown if present)
            content = content.strip()
            if content.startswith("```json"):
                content = content[7:]
            if content.endswith("```"):
                content = content[:-3]
            content = content.strip()
            
            # Parse JSON
            analysis = json.loads(content)
            analysis["api_response"] = result
            analysis["success"] = True
            
            return analysis
            
        except requests.exceptions.RequestException as e:
            return {
                "success": False,
                "error": f"API call failed: {str(e)}",
            }
        except json.JSONDecodeError as e:
            return {
                "success": False,
                "error": f"Failed to parse LLM response as JSON: {str(e)}",
                "raw_response": content if 'content' in dir() else "N/A",
            }
    
    def analyze_package(
        self,
        scan_result_path: str,
        package_dir: str,
    ) -> dict:
        """
        Analyze all findings for a scanned package
        
        Args:
            scan_result_path: Path to glassware JSON output
            package_dir: Path to extracted package
        
        Returns:
            Aggregated analysis results
        """
        # Load scan results
        with open(scan_result_path) as f:
            scan_data = json.load(f)
        
        # Load package.json
        pkg_json_path = Path(package_dir) / "package.json"
        if pkg_json_path.exists():
            with open(pkg_json_path) as f:
                package_info = json.load(f)
        else:
            package_info = {}
        
        # Analyze each finding
        findings = scan_data.get("findings", [])
        analyses = []
        
        print(f"Analyzing {len(findings)} findings...")
        
        for i, finding in enumerate(findings, 1):
            print(f"  [{i}/{len(findings)}] {finding.get('category')} @ line {finding.get('line')}...")
            
            # Get source context (read file around the finding line)
            source_context = self._get_source_context(
                package_dir,
                finding.get("file", ""),
                finding.get("line", 1),
                context_lines=50,
            )
            
            # Analyze
            analysis = self.analyze_finding(finding, package_info, source_context)
            analysis["finding"] = finding
            analyses.append(analysis)
        
        # Aggregate results
        malicious_count = sum(1 for a in analyses if a.get("classification") == "MALICIOUS")
        suspicious_count = sum(1 for a in analyses if a.get("classification") == "SUSPICIOUS")
        fp_count = sum(1 for a in analyses if a.get("classification") == "FALSE_POSITIVE")
        
        return {
            "package_name": package_info.get("name", "unknown"),
            "package_version": package_info.get("version", "unknown"),
            "scan_date": datetime.utcnow().isoformat() + "Z",
            "total_findings": len(findings),
            "malicious_count": malicious_count,
            "suspicious_count": suspicious_count,
            "false_positive_count": fp_count,
            "overall_classification": self._get_overall_classification(
                malicious_count, suspicious_count, fp_count, len(findings)
            ),
            "analyses": analyses,
        }
    
    def _get_source_context(
        self,
        package_dir: str,
        file_path: str,
        line_num: int,
        context_lines: int = 50,
    ) -> str:
        """Extract source code context around a finding"""
        # Handle absolute vs relative paths
        if file_path.startswith(package_dir):
            full_path = Path(file_path)
        else:
            full_path = Path(package_dir) / file_path.replace("package/", "")
        
        if not full_path.exists():
            return "// File not found"
        
        try:
            with open(full_path, errors="ignore") as f:
                lines = f.readlines()
            
            # Get context around the line
            start = max(0, line_num - context_lines // 2)
            end = min(len(lines), line_num + context_lines // 2)
            
            return "".join(lines[start:end])
            
        except Exception as e:
            return f"// Error reading file: {e}"
    
    def _get_overall_classification(
        self,
        malicious: int,
        suspicious: int,
        fp: int,
        total: int,
    ) -> str:
        """Determine overall package classification"""
        if total == 0:
            return "CLEAN"
        
        malicious_ratio = malicious / total
        
        if malicious_ratio > 0.5:
            return "MALICIOUS"
        elif malicious_ratio > 0.2 or suspicious / total > 0.5:
            return "SUSPICIOUS"
        elif fp / total > 0.8:
            return "FALSE_POSITIVE"
        else:
            return "NEEDS_REVIEW"


def main():
    """CLI entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Analyze glassware security findings with LLM"
    )
    parser.add_argument(
        "scan_result",
        help="Path to glassware JSON scan result",
    )
    parser.add_argument(
        "package_dir",
        help="Path to extracted package directory",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="analysis_result.json",
        help="Output file for analysis results",
    )
    
    args = parser.parse_args()
    
    # Initialize analyzer
    try:
        analyzer = NIMAnalyzer()
    except ValueError as e:
        print(f"Error: {e}")
        return 1
    
    # Run analysis
    result = analyzer.analyze_package(args.scan_result, args.package_dir)
    
    # Save results
    with open(args.output, "w") as f:
        json.dump(result, f, indent=2)
    
    # Print summary
    print("\n" + "="*60)
    print("ANALYSIS SUMMARY")
    print("="*60)
    print(f"Package: {result['package_name']}@{result['package_version']}")
    print(f"Total Findings: {result['total_findings']}")
    print(f"  - Malicious: {result['malicious_count']}")
    print(f"  - Suspicious: {result['suspicious_count']}")
    print(f"  - False Positive: {result['false_positive_count']}")
    print(f"Overall Classification: {result['overall_classification']}")
    print(f"Results saved to: {args.output}")
    print("="*60)
    
    return 0


if __name__ == "__main__":
    exit(main())
