#!/usr/bin/env python3
"""
LLM-Based Package Prioritization

Analyzes scan results and suggests which categories/packages to scan next.
Uses LLM to identify high-risk patterns and prioritize accordingly.
"""

import json
import os
import argparse
from pathlib import Path
from datetime import datetime
import requests

NVIDIA_API_KEY = os.environ.get("NVIDIA_API_KEY", "")
MODEL = "meta/llama-3.3-70b-instruct"

# Category definitions
CATEGORIES = {
    "native-build": ["node-gyp", "bindings", "prebuild", "nan", "node-addon-api"],
    "install-scripts": ["preinstall", "postinstall", "install"],
    "web-frameworks": ["react", "vue", "angular", "svelte", "next", "nuxt"],
    "backend": ["express", "fastify", "koa", "hapi", "nest"],
    "database": ["mongoose", "sequelize", "typeorm", "prisma", "knex"],
    "devtools": ["eslint", "prettier", "typescript", "babel", "webpack", "vite"],
    "testing": ["jest", "mocha", "vitest", "cypress", "playwright"],
    "cli": ["commander", "yargs", "chalk", "ora", "inquirer"],
    "crypto": ["crypto", "bcrypt", "jsonwebtoken", "jose", "node-forge"],
    "security": ["helmet", "cors", "rate-limit", "validator", "sanitize"],
    "ai-ml": ["ai", "llm", "openai", "anthropic", "langchain", "agent"],
    "utils": ["lodash", "async", "moment", "dayjs", "axios", "got"],
    "logging": ["winston", "pino", "log4js", "debug"],
}

PRIORITIZATION_PROMPT = """You are a security analyst prioritizing npm packages for GlassWare scanning.

## Current Scan Results

Total packages scanned: {total_scanned}
Flagged packages: {flagged_count}
Flagged rate: {flagged_rate:.1f}%

### Top Flagged Packages
{top_flagged}

### Category Breakdown (from sampling)
{category_breakdown}

## Analysis Tasks

1. **Risk Assessment**: Based on the flagged packages, which categories appear highest risk?

2. **Pattern Recognition**: Do you see any patterns in the flagged packages?
   - Specific publishers?
   - Specific file types?
   - Specific functionality?

3. **Prioritization**: Recommend the next 3-5 categories to scan, in priority order.
   For each category, explain:
   - Why it's high priority
   - Expected risk level (high/medium/low)
   - Suggested sample size

4. **Specific Packages**: Are there any specific packages from the flagged list that need immediate human review?

## Response Format

Respond with ONLY valid JSON (no markdown):

{{
  "risk_assessment": "2-3 sentence summary",
  "patterns_identified": ["pattern1", "pattern2"],
  "recommended_categories": [
    {{
      "category": "category-name",
      "priority": 1,
      "reason": "Why this category is high priority",
      "expected_risk": "high|medium|low",
      "sample_size": 100
    }}
  ],
  "packages_for_review": [
    {{
      "package": "@scope/name@version",
      "reason": "Why this needs review",
      "urgency": "high|medium|low"
    }}
  ],
  "confidence": 0.0-1.0
}}
"""


def analyze_scan_results(scan_results_path: str) -> dict:
    """Analyze scan results and generate prioritization recommendations"""
    
    # Load scan results
    with open(scan_results_path) as f:
        data = json.load(f)
    
    # Extract statistics
    total_scanned = data.get("total_packages", 0)
    flagged = data.get("flagged_packages", [])
    flagged_count = len(flagged)
    flagged_rate = (flagged_count / total_scanned * 100) if total_scanned > 0 else 0
    
    # Format top flagged
    top_flagged = "\n".join([
        f"- {pkg['package']}: {pkg['findings']} findings ({pkg['critical']} critical)"
        for pkg in sorted(flagged, key=lambda x: -x['critical'])[:10]
    ])
    
    # Format category breakdown
    category_breakdown = "\n".join([
        f"- {cat}: ~{len(keywords) * 10} packages"
        for cat, keywords in CATEGORIES.items()
    ])
    
    # Build prompt
    prompt = PRIORITIZATION_PROMPT.format(
        total_scanned=total_scanned,
        flagged_count=flagged_count,
        flagged_rate=flagged_rate,
        top_flagged=top_flagged,
        category_breakdown=category_breakdown,
    )
    
    # Call LLM
    if not NVIDIA_API_KEY:
        raise ValueError("NVIDIA_API_KEY not set")
    
    payload = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": "You are a security analyst. Respond with ONLY valid JSON."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1,
        "max_tokens": 2048,
    }
    
    headers = {
        "Authorization": f"Bearer {NVIDIA_API_KEY}",
        "Content-Type": "application/json",
    }
    
    response = requests.post(
        "https://integrate.api.nvidia.com/v1/chat/completions",
        headers=headers,
        json=payload,
        timeout=120,
    )
    response.raise_for_status()
    
    result = response.json()
    content = result["choices"][0]["message"]["content"]
    
    # Parse JSON response
    content = content.strip()
    if content.startswith("```json"):
        content = content[7:]
    if content.endswith("```"):
        content = content[:-3]
    content = content.strip()
    
    return json.loads(content)


def generate_package_list(recommendations: dict, output_file: str) -> None:
    """Generate a package list file based on recommendations"""
    
    # For now, just note the recommendations
    # In future, this would call diverse_sampling.py with recommended categories
    
    print(f"Recommended categories for next scan:")
    for cat in recommendations.get("recommended_categories", []):
        print(f"  {cat['priority']}. {cat['category']} ({cat['expected_risk']} risk) - {cat['sample_size']} samples")
        print(f"     Reason: {cat['reason']}")
    
    print(f"\nPackages for immediate review:")
    for pkg in recommendations.get("packages_for_review", []):
        print(f"  - {pkg['package']} ({pkg['urgency']} urgency)")
        print(f"    Reason: {pkg['reason']}")


def main():
    parser = argparse.ArgumentParser(description="LLM-based package prioritization")
    parser.add_argument("scan_results", help="Path to scan results JSON")
    parser.add_argument("--output", "-o", default="prioritized-packages.txt", help="Output file")
    parser.add_argument("--output-json", default="prioritization-results.json", help="JSON output file")
    
    args = parser.parse_args()
    
    print("="*70)
    print("LLM PACKAGE PRIORITIZATION")
    print("="*70)
    print(f"Analyzing: {args.scan_results}")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    # Analyze
    recommendations = analyze_scan_results(args.scan_results)
    
    # Save JSON results
    with open(args.output_json, "w") as f:
        json.dump(recommendations, f, indent=2)
    
    # Generate package list
    generate_package_list(recommendations, args.output)
    
    print()
    print("="*70)
    print("PRIORITIZATION COMPLETE")
    print("="*70)
    print(f"Confidence: {recommendations.get('confidence', 0):.0%}")
    print(f"Results saved to: {args.output_json}")
    print(f"Package list: {args.output}")
    print("="*70)


if __name__ == "__main__":
    main()
