# LLM Orchestration Design Document

**Date:** 2026-03-19  
**Status:** Design Proposal  
**Priority:** High (Phase 3 Implementation)  

---

## Executive Summary

**Goal:** Build a modular, provider-agnostic LLM orchestration layer that:
1. Supports multiple LLM providers (NVIDIA, Cerebras, Groq, OpenAI, etc.)
2. Respects individual rate limits per provider/model
3. Automatically fails over between providers
4. Optimizes for cost, speed, and quality
5. Is easy to extend with new providers/models

**Design Principles:**
- **Modular:** Each provider is a pluggable module
- **Configurable:** All settings via YAML config
- **Observable:** Full logging and metrics
- **Resilient:** Graceful degradation on failures
- **Elegant:** Simple interfaces, clear separation of concerns

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    LLM Orchestrator                          │
├─────────────────────────────────────────────────────────────┤
│  Provider Registry                                           │
│  ├── NVIDIA Provider (llama-3.3-70b, etc.)                  │
│  ├── Cerebras Provider (llama-3.3-70b)                      │
│  ├── Groq Provider (llama-3.3-70b-versatile)                │
│  └── OpenAI Provider (gpt-4o, etc.)                         │
├─────────────────────────────────────────────────────────────┤
│  Rate Limiter (Token Bucket per Provider)                   │
├─────────────────────────────────────────────────────────────┤
│  Load Balancer (Round-robin / Priority / Cost-based)        │
├─────────────────────────────────────────────────────────────┤
│  Retry Logic (Exponential backoff, circuit breaker)         │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌──────────┐        ┌──────────┐        ┌──────────┐
   │ NVIDIA   │        │ Cerebras │        │   Groq   │
   │   NIM    │        │   API    │        │   API    │
   └──────────┘        └──────────┘        └──────────┘
```

---

## Core Interfaces

### 1. Provider Interface (Abstract Base Class)

```python
# llm/providers/base.py
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional
from collections import deque
import time

@dataclass
class RateLimitConfig:
    """Rate limit configuration for a provider"""
    requests_per_minute: int
    tokens_per_minute: int
    max_retries: int = 3
    base_backoff_seconds: float = 2.0

@dataclass
class ModelConfig:
    """Model configuration"""
    name: str
    context_window: int  # Max tokens
    cost_per_1k_tokens: float = 0.0  # For cost optimization
    speed_tier: str = "standard"  # fast, standard, slow

class LLMProvider(ABC):
    """Abstract base class for all LLM providers"""
    
    def __init__(
        self,
        api_key: str,
        base_url: str,
        rate_limit: RateLimitConfig,
        default_model: ModelConfig,
    ):
        self.api_key = api_key
        self.base_url = base_url
        self.rate_limit = rate_limit
        self.default_model = default_model
        
        # Rate limiting state
        self.request_times = deque()
        self.token_count = 0
        self.token_window_start = time.time()
        
        # Metrics
        self.total_requests = 0
        self.successful_requests = 0
        self.failed_requests = 0
        self.total_tokens = 0
    
    @abstractmethod
    def analyze(
        self,
        finding: dict,
        package_info: dict,
        source_context: str,
    ) -> dict:
        """
        Analyze a security finding and return classification.
        
        Returns:
            {
                "classification": "MALICIOUS" | "SUSPICIOUS" | "FALSE_POSITIVE",
                "confidence": 0.0-1.0,
                "reasoning": "...",
                "indicators": [...],
                "recommended_action": "...",
            }
        """
        pass
    
    def wait_if_needed(self, estimated_tokens: int = 1000):
        """Wait if rate limit would be exceeded"""
        now = time.time()
        
        # Clean old request times (older than 1 minute)
        while self.request_times and now - self.request_times[0] >= 60:
            self.request_times.popleft()
        
        # Check RPM limit
        if len(self.request_times) >= self.rate_limit.requests_per_minute:
            sleep_time = 60 - (now - self.request_times[0])
            if sleep_time > 0:
                time.sleep(sleep_time)
        
        # Reset token count every minute
        if now - self.token_window_start >= 60:
            self.token_count = 0
            self.token_window_start = now
        
        # Check TPM limit
        if self.token_count + estimated_tokens > self.rate_limit.tokens_per_minute:
            sleep_time = 60 - (now - self.token_window_start)
            if sleep_time > 0:
                time.sleep(sleep_time)
            self.token_count = 0
            self.token_window_start = time.time()
        
        # Record this request
        self.request_times.append(time.time())
        self.token_count += estimated_tokens
    
    def get_metrics(self) -> dict:
        """Get provider metrics"""
        return {
            "total_requests": self.total_requests,
            "successful_requests": self.successful_requests,
            "failed_requests": self.failed_requests,
            "success_rate": self.successful_requests / max(1, self.total_requests),
            "total_tokens": self.total_tokens,
            "current_rpm": len(self.request_times),
            "current_tpm": self.token_count,
        }
```

---

### 2. Concrete Provider Implementations

```python
# llm/providers/nvidia.py
from .base import LLMProvider, RateLimitConfig, ModelConfig

class NVIDIAProvider(LLMProvider):
    """NVIDIA NIM provider"""
    
    def __init__(self, api_key: str):
        super().__init__(
            api_key=api_key,
            base_url="https://integrate.api.nvidia.com/v1",
            rate_limit=RateLimitConfig(
                requests_per_minute=30,
                tokens_per_minute=60000,
            ),
            default_model=ModelConfig(
                name="meta/llama-3.3-70b-instruct",
                context_window=131072,
                cost_per_1k_tokens=0.0,  # Free tier
                speed_tier="standard",
            ),
        )
        
        # Available models on NVIDIA NIM
        self.available_models = {
            "llama-3.3-70b": ModelConfig(
                name="meta/llama-3.3-70b-instruct",
                context_window=131072,
                speed_tier="standard",
            ),
            "llama-3.1-405b": ModelConfig(
                name="meta/llama-3.1-405b-instruct",
                context_window=131072,
                speed_tier="slow",
            ),
            "mistral-large": ModelConfig(
                name="mistralai/mistral-large-2-instruct",
                context_window=131072,
                speed_tier="fast",
            ),
        }
    
    def analyze(self, finding, package_info, source_context):
        # Implementation using requests.post to NVIDIA NIM API
        pass

# llm/providers/cerebras.py
class CerebrasProvider(LLMProvider):
    """Cerebras provider"""
    
    def __init__(self, api_key: str):
        super().__init__(
            api_key=api_key,
            base_url="https://api.cerebras.ai/v1",
            rate_limit=RateLimitConfig(
                requests_per_minute=60,
                tokens_per_minute=100000,
            ),
            default_model=ModelConfig(
                name="llama-3.3-70b",
                context_window=131072,
                cost_per_1k_tokens=0.0,
                speed_tier="fast",
            ),
        )

# llm/providers/groq.py
class GroqProvider(LLMProvider):
    """Groq provider"""
    
    def __init__(self, api_key: str):
        super().__init__(
            api_key=api_key,
            base_url="https://api.groq.com/openai/v1",
            rate_limit=RateLimitConfig(
                requests_per_minute=30,
                tokens_per_minute=60000,
            ),
            default_model=ModelConfig(
                name="llama-3.3-70b-versatile",
                context_window=131072,
                cost_per_1k_tokens=0.0,
                speed_tier="fast",
            ),
        )
```

---

### 3. Orchestrator (Load Balancer + Failover)

```python
# llm/orchestrator.py
from typing import List, Optional
from .providers.base import LLMProvider, ModelConfig
import random

class LoadBalancerStrategy:
    """Load balancing strategy"""
    ROUND_ROBIN = "round_robin"
    PRIORITY = "priority"
    LEAST_LOADED = "least_loaded"
    COST_OPTIMIZED = "cost_optimized"
    SPEED_OPTIMIZED = "speed_optimized"

class LLMOchestrator:
    """
    Orchestrates multiple LLM providers with automatic failover.
    """
    
    def __init__(
        self,
        providers: List[LLMProvider],
        strategy: LoadBalancerStrategy = LoadBalancerStrategy.PRIORITY,
        provider_priorities: Optional[dict[str, int]] = None,
    ):
        self.providers = {p.__class__.__name__: p for p in providers}
        self.strategy = strategy
        self.priorities = provider_priorities or {}
        self.current_index = 0  # For round-robin
    
    def select_provider(self) -> LLMProvider:
        """Select next provider based on strategy"""
        if self.strategy == LoadBalancerStrategy.ROUND_ROBIN:
            return self._select_round_robin()
        elif self.strategy == LoadBalancerStrategy.PRIORITY:
            return self._select_priority()
        elif self.strategy == LoadBalancerStrategy.LEAST_LOADED:
            return self._select_least_loaded()
        elif self.strategy == LoadBalancerStrategy.COST_OPTIMIZED:
            return self._select_cost_optimized()
        elif self.strategy == LoadBalancerStrategy.SPEED_OPTIMIZED:
            return self._select_speed_optimized()
        else:
            return list(self.providers.values())[0]
    
    def _select_round_robin(self) -> LLMProvider:
        """Round-robin selection"""
        provider_list = list(self.providers.values())
        provider = provider_list[self.current_index % len(provider_list)]
        self.current_index += 1
        return provider
    
    def _select_priority(self) -> LLMProvider:
        """Select by priority (lowest number = highest priority)"""
        sorted_providers = sorted(
            self.providers.values(),
            key=lambda p: self.priorities.get(p.__class__.__name__, 999)
        )
        return sorted_providers[0]
    
    def _select_least_loaded(self) -> LLMProvider:
        """Select provider with lowest current load"""
        return min(
            self.providers.values(),
            key=lambda p: p.get_metrics()["current_rpm"] / p.rate_limit.requests_per_minute
        )
    
    def _select_cost_optimized(self) -> LLMProvider:
        """Select cheapest provider"""
        return min(
            self.providers.values(),
            key=lambda p: p.default_model.cost_per_1k_tokens
        )
    
    def _select_speed_optimized(self) -> LLMProvider:
        """Select fastest provider"""
        speed_order = {"fast": 0, "standard": 1, "slow": 2}
        return min(
            self.providers.values(),
            key=lambda p: speed_order.get(p.default_model.speed_tier, 999)
        )
    
    def analyze(
        self,
        finding: dict,
        package_info: dict,
        source_context: str,
        max_retries: int = 3,
    ) -> dict:
        """
        Analyze finding with automatic failover.
        
        Tries providers in order until one succeeds.
        """
        last_error = None
        
        for attempt in range(max_retries):
            provider = self.select_provider()
            
            try:
                # Wait for rate limit
                provider.wait_if_needed()
                
                # Make request
                result = provider.analyze(finding, package_info, source_context)
                
                # Record success
                provider.successful_requests += 1
                return result
                
            except RateLimitError as e:
                # Rate limited - try next provider
                provider.failed_requests += 1
                last_error = e
                continue
                
            except Exception as e:
                # Other error - retry with same provider
                provider.failed_requests += 1
                last_error = e
                
                if attempt < max_retries - 1:
                    # Exponential backoff
                    wait_time = provider.rate_limit.base_backoff_seconds * (2 ** attempt)
                    time.sleep(wait_time)
        
        # All providers failed
        raise OrchestrationError(
            f"All providers failed after {max_retries} attempts",
            last_error,
        )
    
    def get_all_metrics(self) -> dict:
        """Get metrics from all providers"""
        return {
            name: provider.get_metrics()
            for name, provider in self.providers.items()
        }
```

---

## Configuration

### YAML Configuration File

```yaml
# llm-config.yaml
orchestrator:
  strategy: priority  # round_robin, priority, least_loaded, cost_optimized, speed_optimized
  
providers:
  - name: nvidia
    type: nvidia
    api_key_env: NVIDIA_API_KEY
    priority: 1
    models:
      - name: llama-3.3-70b
        context_window: 131072
        speed_tier: standard
      - name: llama-3.1-405b
        context_window: 131072
        speed_tier: slow
    rate_limit:
      requests_per_minute: 30
      tokens_per_minute: 60000
    
  - name: cerebras
    type: cerebras
    api_key_env: CEREBRAS_API_KEY
    priority: 2
    models:
      - name: llama-3.3-70b
        context_window: 131072
        speed_tier: fast
    rate_limit:
      requests_per_minute: 60
      tokens_per_minute: 100000
    
  - name: groq
    type: groq
    api_key_env: GROQ_API_KEY
    priority: 3
    models:
      - name: llama-3.3-70b-versatile
        context_window: 131072
        speed_tier: fast
    rate_limit:
      requests_per_minute: 30
      tokens_per_minute: 60000

fallback:
  - nvidia
  - cerebras
  - groq

batch_analysis:
  workers: 3
  max_tokens_per_request: 1000
  timeout_seconds: 120
```

---

## Usage Examples

### Basic Usage

```python
from llm.orchestrator import LLMOchestrator, LoadBalancerStrategy
from llm.providers.nvidia import NVIDIAProvider
from llm.providers.cerebras import CerebrasProvider
from llm.providers.groq import GroqProvider

# Initialize providers
nvidia = NVIDIAProvider(api_key="nvapi-...")
cerebras = CerebrasProvider(api_key="csk-...")
groq = GroqProvider(api_key="gsk-...")

# Create orchestrator with priority-based failover
orchestrator = LLMOchestrator(
    providers=[nvidia, cerebras, groq],
    strategy=LoadBalancerStrategy.PRIORITY,
    provider_priorities={
        "NVIDIAProvider": 1,
        "CerebrasProvider": 2,
        "GroqProvider": 3,
    },
)

# Analyze a finding
result = orchestrator.analyze(
    finding=finding_dict,
    package_info=package_info_dict,
    source_context=source_code,
)

print(f"Classification: {result['classification']}")
print(f"Confidence: {result['confidence']}")
```

### Load from Config

```python
from llm.config import load_config, create_orchestrator_from_config

# Load configuration
config = load_config("llm-config.yaml")

# Create orchestrator
orchestrator = create_orchestrator_from_config(config)

# Use as normal
result = orchestrator.analyze(...)
```

### Batch Analysis

```python
from concurrent.futures import ThreadPoolExecutor

def analyze_batch(orchestrator, findings_batch):
    """Analyze batch of findings in parallel"""
    with ThreadPoolExecutor(max_workers=3) as executor:
        futures = [
            executor.submit(
                orchestrator.analyze,
                finding=f,
                package_info=package_info,
                source_context=context,
            )
            for f, package_info, context in findings_batch
        ]
        results = [f.result() for f in futures]
    return results
```

---

## Metrics & Observability

### Provider Metrics

```python
metrics = orchestrator.get_all_metrics()

# Example output:
{
    "NVIDIAProvider": {
        "total_requests": 100,
        "successful_requests": 95,
        "failed_requests": 5,
        "success_rate": 0.95,
        "total_tokens": 50000,
        "current_rpm": 25,
        "current_tpm": 45000,
    },
    "CerebrasProvider": {...},
    "GroqProvider": {...},
}
```

### Logging

```python
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("llm.orchestrator")

# Log provider selection
logger.info(f"Selected provider: {provider.__class__.__name__}")

# Log rate limit waits
logger.debug(f"Waiting {sleep_time}s for rate limit")

# Log failures
logger.error(f"Provider {name} failed: {error}")
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (2 days)
- [ ] Base provider interface
- [ ] NVIDIA provider implementation
- [ ] Basic orchestrator with priority failover
- [ ] Configuration loading

### Phase 2: Additional Providers (1 day)
- [ ] Cerebras provider
- [ ] Groq provider
- [ ] Test failover between providers

### Phase 3: Advanced Features (2 days)
- [ ] Load balancing strategies (round-robin, least-loaded)
- [ ] Cost optimization
- [ ] Speed optimization
- [ ] Batch analysis with parallel execution

### Phase 4: Observability (1 day)
- [ ] Metrics collection
- [ ] Logging integration
- [ ] Dashboard/monitoring

**Total:** 6 days for full implementation

---

## Recommendations

### Start Simple
1. **Week 1:** Implement Phase 1 + 2 (NVIDIA + Cerebras with failover)
2. **Week 2:** Test in production, gather metrics
3. **Week 3:** Add advanced features based on real usage patterns

### Keep It Modular
- Each provider is independent
- Easy to add new providers
- Configuration-driven, not hardcoded

### Monitor Everything
- Track success rates per provider
- Track latency per provider
- Track costs per provider
- Use data to optimize strategy

---

**Status:** Ready for implementation  
**Next Step:** Review design, approve, then implement Phase 1
