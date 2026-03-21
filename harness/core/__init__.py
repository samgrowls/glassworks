"""
GlassWorm Harness Core

Core infrastructure for package scanning, analysis, and reporting.

Modules:
- store: SQLite database with checkpoint/resume support
- fetcher: npm package download and caching
- scanner: GlassWorm CLI wrapper
- analyzer: NVIDIA LLM deep analysis
- reporter: Markdown and JSON report generation
- orchestrator: Wave-based pipeline runner
"""

from .store import Store, get_connection, init_database
from .fetcher import Fetcher, NPM_REGISTRY
from .scanner import Scanner, GLASSWARE_CLI
from .analyzer import Analyzer, NVIDIA_API_KEY
from .reporter import Reporter
from .orchestrator import Orchestrator

__all__ = [
    "Store",
    "get_connection",
    "init_database",
    "Fetcher",
    "NPM_REGISTRY",
    "Scanner",
    "GLASSWARE_CLI",
    "Analyzer",
    "NVIDIA_API_KEY",
    "Reporter",
    "Orchestrator",
]

__version__ = "0.1.0"
