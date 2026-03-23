# Glassworks Campaign System Architecture

**Version:** 1.1.0
**Date:** March 22, 2026
**Status:** Architecture Specification - Approved for Implementation

**Changes in v1.1.0:**
- Added TUI architecture support (RFC-001)
- Event bus design for real-time monitoring
- Command channel for campaign steering
- State manager for queryable campaign state

---

## 1. Executive Summary

This document specifies the architecture for the **Glassworks Campaign System** - a robust, production-grade wave campaign orchestration system for the Rust orchestrator. The design incorporates best practices from leading security scanning tools (cargo-audit, trufflehog, semgrep, grype) and forensic evidence collection standards (NIST SP 800-86, ISO 27037).

### Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Rust-first** | Rust orchestrator is primary; Python is legacy |
| **Enhanced campaigns** | Flexible TOML config over Python's waves.toml |
| **Evidence-ready** | Court-admissible evidence collection from day one |
| **Checkpoint everything** | Long-running scans must resume cleanly |
| **Progress visibility** | Real-time ETA, percentages, MultiProgress UI |
| **Dual-mode** | Simple single-package scans AND large campaigns |

---

## 2. System Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    glassware-orchestrator CLI                    │
│                                                                  │
│  clap v4 (CLI parsing)                                           │
│  ├── campaign run <config.toml>                                  │
│  ├── campaign resume <case_id>                                   │
│  ├── campaign status <case_id>                                   │
│  ├── campaign report <case_id> [--format markdown|sarif|json]   │
│  ├── scan-npm <packages...>                                      │
│  ├── scan-github <repos...>                                      │
│  ├── scan-file <file.txt>                                        │
│  └── scan-tarball <files...>                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Campaign Orchestrator                         │
├─────────────────────────────────────────────────────────────────┤
│  CampaignConfig (TOML parsing + validation)                      │
│  ├── Schema validation with serde                                │
│  ├── Default values and overrides                                │
│  └── Wave dependency graph (DAG)                                 │
│                                                                  │
│  WaveExecutor (tokio async runtime)                              │
│  ├── DAG scheduler (parallel waves where possible)               │
│  ├── Worker pool (configurable concurrency)                      │
│  ├── Rate limiter (governor crate)                               │
│  └── Cancellation token (tokio-util)                             │
│                                                                  │
│  CheckpointManager (SQLite via sqlx)                             │
│  ├── Transactional checkpoints                                   │
│  ├── Wave state persistence                                      │
│  └── Resume capability                                           │
│                                                                  │
│  ProgressReporter (indicatif + tracing-indicatif)                │
│  ├── MultiProgress for concurrent waves                          │
│  ├── ETA calculation                                             │
│  └── Tracing span integration                                    │
│                                                                  │
│  EvidenceCollector                                               │
│  ├── SHA-256 integrity verification                              │
│  ├── Chain of custody logging                                    │
│  └── Evidence manifest generation                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Scan Engine                                 │
│  (Existing glassware-core detectors)                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Report Generator                            │
├─────────────────────────────────────────────────────────────────┤
│  MarkdownReport (programmatic + tera templates)                  │
│  ├── Executive summary                                           │
│  ├── Wave results                                                │
│  ├── LLM analysis summary                                        │
│  ├── Findings by category                                        │
│  └── Evidence manifest                                           │
│                                                                  │
│  SarifReport (GitHub Advanced Security)                          │
│  └── SARIF 2.1.0 compliant output                                │
│                                                                  │
│  JsonReport (machine-readable)                                   │
│  └── Full structured results                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Campaign Configuration Model

### 3.1 Configuration Structure

```toml
# campaigns/wave6.toml

[campaign]
name = "Wave 6: React Native Hunt"
description = "Targeted hunting in React Native ecosystem for GlassWorm patterns"
created_by = "security-team"
priority = "high"
tags = ["react-native", "glassworm", "unicode-stego"]

# Global campaign settings
[settings]
concurrency = 15
rate_limit_npm = 15.0
rate_limit_github = 5.0
cache_enabled = true
cache_ttl_days = 7

# LLM configuration
[settings.llm]
tier1_enabled = true                    # Cerebras during scan
tier1_provider = "cerebras"
tier2_enabled = true                    # NVIDIA for flagged
tier2_threshold = 5.0                   # threat_score >= 5.0
tier2_models = [                        # Fallback chain
    "qwen/qwen3.5-397b-a17b",
    "moonshotai/kimi-k2.5",
    "z-ai/glm5",
    "meta/llama-3.3-70b-instruct"
]

# Output configuration
[settings.output]
formats = ["json", "markdown", "sarif"]
evidence_collection = true
evidence_dir = "evidence"
report_dir = "reports"

# Wave definitions
[[waves]]
id = "wave_6a"
name = "Known Malicious Baseline"
description = "Validate detection with confirmed malicious packages"
depends_on = []
mode = "validate"  # validate | hunt | monitor

[[waves.sources]]
type = "packages"
list = [
    "react-native-country-select@0.3.91",
    "react-native-international-phone-number@0.11.8",
]

# Validation expectations
[waves.expectations]
must_flag_all = true
min_threat_score = 7.0

[[waves]]
id = "wave_6b"
name = "React Native Ecosystem"
description = "Hunt for GlassWorm in React Native packages"
depends_on = ["wave_6a"]  # Runs after 6a completes
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = [
    "react-native-phone",
    "react-native-country",
    "react-native-locale",
    "react-native-i18n"
]
samples_per_keyword = 30
days_recent = 365
max_downloads = 10000  # Focus on less popular packages

[[waves.sources]]
type = "npm_category"
category = "react-native"
samples = 50
sort_by = "recent"  # recent | popular | random

# Per-wave whitelist
[[waves.whitelist]]
packages = ["react-native", "react-native-maps", "react-native-vector-icons"]
reason = "Well-known, verified clean"

[[waves]]
id = "wave_6c"
name = "MCP/AI Infrastructure"
description = "Scan AI/LLM infrastructure packages"
depends_on = []  # Can run parallel with 6b
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["mcp", "llm", "langchain", "agent", "ai-sdk"]
samples_per_keyword = 25

# Reporting overrides
[waves.reporting]
include_clean_summary = false
slack_webhook = "https://hooks.slack.com/..."
```

### 3.2 Configuration Schema

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignConfig {
    pub campaign: CampaignMetadata,
    pub settings: CampaignSettings,
    pub waves: Vec<WaveConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignMetadata {
    pub name: String,
    pub description: String,
    pub created_by: String,
    pub priority: Priority,  // low | medium | high | critical
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignSettings {
    pub concurrency: usize,
    pub rate_limit_npm: f32,
    pub rate_limit_github: f32,
    pub cache_enabled: bool,
    pub cache_ttl_days: u64,
    pub llm: LlmSettings,
    pub output: OutputSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WaveConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub depends_on: Vec<String>,
    pub mode: WaveMode,  // validate | hunt | monitor
    pub sources: Vec<WaveSource>,
    pub whitelist: Option<WhitelistConfig>,
    pub expectations: Option<ValidationExpectations>,
    pub reporting: Option<ReportingOverrides>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum WaveSource {
    #[serde(rename = "packages")]
    Packages { list: Vec<String> },
    
    #[serde(rename = "npm_search")]
    NpmSearch {
        keywords: Vec<String>,
        samples_per_keyword: usize,
        days_recent: Option<u32>,
        max_downloads: Option<u64>,
    },
    
    #[serde(rename = "npm_category")]
    NpmCategory {
        category: String,
        samples: usize,
        sort_by: SortOrder,  // recent | popular | random
    },
    
    #[serde(rename = "github_search")]
    GitHubSearch {
        query: String,
        max_results: usize,
        sort_by: GitHubSort,  // stars | forks | updated
    },
}
```

---

## 4. Wave Execution Engine

### 4.1 DAG Scheduler

```rust
pub struct DagScheduler {
    waves: HashMap<String, WaveConfig>,
    graph: DependencyGraph,
}

impl DagScheduler {
    /// Build execution plan from wave dependencies
    pub fn build_execution_plan(&self) -> Vec<ExecutionStage> {
        // Topological sort with parallel grouping
        let mut stages = Vec::new();
        let mut completed = HashSet::new();
        let mut remaining: HashSet<_> = self.waves.keys().cloned().collect();
        
        while !remaining.is_empty() {
            // Find all waves that can run now (dependencies met)
            let ready: Vec<_> = remaining.iter()
                .filter(|wave_id| {
                    let wave = &self.waves[*wave_id];
                    wave.depends_on.iter().all(|dep| completed.contains(dep))
                })
                .cloned()
                .collect();
            
            if ready.is_empty() {
                panic!("Circular dependency detected!");
            }
            
            stages.push(ExecutionStage {
                waves: ready.clone(),
                parallel: true,  // All waves in stage can run in parallel
            });
            
            for wave_id in &ready {
                completed.insert(wave_id.clone());
                remaining.remove(wave_id);
            }
        }
        
        stages
    }
}

pub struct ExecutionStage {
    pub waves: Vec<String>,
    pub parallel: bool,
}
```

### 4.2 Worker Pool

```rust
pub struct WorkerPool {
    concurrency: usize,
    semaphore: Arc<Semaphore>,
    rate_limiter: RateLimiter,
}

impl WorkerPool {
    pub fn new(concurrency: usize, rate_limit: f32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(rate_limit as u32).unwrap());
        let rate_limiter = RateLimiter::direct(quota);
        
        Self {
            concurrency,
            semaphore: Arc::new(Semaphore::new(concurrency)),
            rate_limiter,
        }
    }
    
    pub async fn scan_package(&self, package: &PackageSpec) -> Result<ScanResult> {
        // Acquire worker slot
        let _permit = self.semaphore.acquire().await?;
        
        // Wait for rate limit
        self.rate_limiter.until_ready().await;
        
        // Perform scan
        scan_package_impl(package).await
    }
}
```

### 4.3 Cancellation & Graceful Shutdown

```rust
use tokio_util::sync::CancellationToken;

pub struct CampaignExecutor {
    cancel_token: CancellationToken,
    checkpoint_manager: CheckpointManager,
}

impl CampaignExecutor {
    pub async fn run(&self, campaign: &CampaignConfig) -> Result<CampaignResult> {
        let child_token = self.cancel_token.child_token();
        
        // Spawn wave tasks
        let mut wave_handles = Vec::new();
        for stage in self.build_execution_plan(campaign) {
            if stage.parallel {
                // Run all waves in stage concurrently
                let handles: Vec<_> = stage.waves.iter()
                    .map(|wave_id| self.spawn_wave(wave_id, child_token.clone()))
                    .collect();
                
                // Wait for all to complete
                let results = futures::future::join_all(handles).await;
                wave_handles.extend(results);
            } else {
                // Run waves sequentially
                for wave_id in &stage.waves {
                    let result = self.spawn_wave(wave_id, child_token.clone()).await?;
                    wave_handles.push(Ok(result));
                }
            }
            
            // Save checkpoint after each stage
            self.checkpoint_manager.save_stage_checkpoint(&stage).await?;
        }
        
        Ok(CampaignResult { waves: wave_handles })
    }
    
    /// Handle Ctrl+C gracefully
    pub async fn wait_for_interrupt(&self) {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        eprintln!("\n⚠️  Interrupt received, saving checkpoint...");
        self.cancel_token.cancel();
        self.checkpoint_manager.save_checkpoint().await;
        eprintln!("✓ Checkpoint saved. Resume with: glassware-orchestrator campaign resume <case_id>");
    }
}
```

---

## 5. Checkpoint/Resume System

### 5.1 Checkpoint Schema (SQLite)

```sql
-- Campaign runs table
CREATE TABLE campaign_runs (
    case_id TEXT PRIMARY KEY,
    campaign_name TEXT NOT NULL,
    config_hash TEXT NOT NULL,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    status TEXT NOT NULL,  -- running | completed | failed | cancelled
    current_stage INTEGER DEFAULT 0,
    total_stages INTEGER NOT NULL
);

-- Wave state table
CREATE TABLE wave_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT NOT NULL,
    wave_id TEXT NOT NULL,
    status TEXT NOT NULL,  -- pending | running | completed | failed
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    packages_total INTEGER,
    packages_scanned INTEGER DEFAULT 0,
    packages_flagged INTEGER DEFAULT 0,
    packages_malicious INTEGER DEFAULT 0,
    error_message TEXT,
    FOREIGN KEY (case_id) REFERENCES campaign_runs(case_id)
);

-- Package scan results
CREATE TABLE package_scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT NOT NULL,
    wave_id TEXT NOT NULL,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    threat_score REAL,
    is_malicious BOOLEAN DEFAULT FALSE,
    findings_count INTEGER DEFAULT 0,
    llm_verdict TEXT,  -- JSON
    scanned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    evidence_path TEXT,
    FOREIGN KEY (case_id) REFERENCES campaign_runs(case_id)
);

-- Index for fast lookups
CREATE INDEX idx_package_scans_case ON package_scans(case_id);
CREATE INDEX idx_wave_state_case ON wave_state(case_id, wave_id);
```

### 5.2 Checkpoint Manager

```rust
pub struct CheckpointManager {
    db: SqlitePool,
    case_id: String,
}

impl CheckpointManager {
    /// Save campaign state transactionally
    pub async fn save_checkpoint(&self, state: &CampaignState) -> Result<()> {
        let mut tx = self.db.begin().await?;
        
        // Update campaign run status
        sqlx::query!(
            r#"
            UPDATE campaign_runs
            SET status = ?, current_stage = ?, completed_at = ?
            WHERE case_id = ?
            "#,
            state.status.as_str(),
            state.current_stage,
            state.completed_at,
            self.case_id
        )
        .execute(&mut *tx)
        .await?;
        
        // Update wave states
        for wave in &state.waves {
            sqlx::query!(
                r#"
                INSERT OR REPLACE INTO wave_state
                (case_id, wave_id, status, packages_total, packages_scanned, packages_flagged, packages_malicious)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                self.case_id,
                wave.id,
                wave.status.as_str(),
                wave.packages_total,
                wave.packages_scanned,
                wave.packages_flagged,
                wave.packages_malicious
            )
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }
    
    /// Resume from checkpoint
    pub async fn load_checkpoint(&self, case_id: &str) -> Result<Option<CampaignState>> {
        let run = sqlx::query_as!(
            CampaignRunRow,
            r#"SELECT * FROM campaign_runs WHERE case_id = ?"#,
            case_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        match run {
            Some(run) if run.status == "running" => {
                // Load wave states
                let waves = sqlx::query_as!(
                    WaveStateRow,
                    r#"SELECT * FROM wave_state WHERE case_id = ?"#,
                    case_id
                )
                .fetch_all(&self.db)
                .await?;
                
                Ok(Some(CampaignState::from_rows(run, waves)))
            }
            _ => Ok(None),
        }
    }
}
```

---

## 6. Progress Reporting

### 6.1 MultiProgress UI

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ProgressUI {
    multi: MultiProgress,
    campaign_pb: ProgressBar,
    wave_pbs: HashMap<String, ProgressBar>,
}

impl ProgressUI {
    pub fn new(campaign_name: &str, total_waves: usize) -> Self {
        let multi = MultiProgress::new();
        
        // Campaign-level progress
        let campaign_pb = multi.add(ProgressBar::new(total_waves as u64));
        campaign_pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] Campaign: {msg} {bar:40.green} {pos}/{len} waves"
            ).unwrap()
        );
        campaign_pb.set_message(campaign_name);
        
        Self {
            multi,
            campaign_pb,
            wave_pbs: HashMap::new(),
        }
    }
    
    pub fn add_wave_progress(&mut self, wave_id: &str, wave_name: &str, total_packages: u64) {
        let pb = self.multi.add(ProgressBar::new(total_packages));
        pb.set_style(
            ProgressStyle::with_template(
                &format!(
                    "[{{elapsed_precise}}] {} {{bar:30.cyan/blue}} {{pos}}/{{len}} ({{eta}})",
                    wave_name
                )
            ).unwrap()
        );
        self.wave_pbs.insert(wave_id.to_string(), pb);
    }
    
    pub fn update_wave(&self, wave_id: &str, scanned: u64) {
        if let Some(pb) = self.wave_pbs.get(wave_id) {
            pb.set_position(scanned);
        }
    }
}
```

### 6.2 Tracing Integration

```rust
use tracing_indicatif::span_ext;

#[tracing::instrument(skip_all, fields(wave_id = %wave.id))]
async fn execute_wave(&self, wave: &WaveConfig) -> Result<WaveResult> {
    let span = Span::current();
    span.pb_set_style(&ProgressStyle::with_template(
        "[{elapsed_precise}] {msg} {bar:30.cyan/blue} {pos}/{len}"
    ).unwrap());
    span.pb_set_message(&wave.name);
    span.pb_set_length(wave.packages.len() as u64);
    
    // ... wave execution ...
    
    span.pb_finish();
    Ok(result)
}
```

---

## 7. Evidence Collection System

### 7.1 Evidence Directory Structure

```
evidence/
└── {case_id}_{timestamp}/
    ├── metadata/
    │   ├── case_info.json
    │   ├── chain_of_custody.json
    │   └── evidence_manifest.json
    │
    ├── packages/
    │   └── {package_name}_{version}/
    │       ├── package.tgz          # Original tarball
    │       ├── package.tgz.sha256   # Integrity hash
    │       ├── scan_results.json    # Full scan output
    │       ├── llm_analysis.json    # LLM verdicts
    │       └── metadata.json        # Collection metadata
    │
    └── reports/
        ├── preliminary_report.md
        └── final_report.md
```

### 7.2 Evidence Metadata Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceManifest {
    pub case_id: String,
    pub generated_at: DateTime<Utc>,
    pub tool: String,
    pub items: Vec<EvidenceItem>,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub evidence_type: String,  // npm_package | github_repo
    pub source: PackageSource,
    pub collection: CollectionMetadata,
    pub integrity: IntegrityMetadata,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub timestamp: DateTime<Utc>,
    pub method: String,  // npm_registry | github_api
    pub collector: String,
    pub tool: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityMetadata {
    pub algorithm: String,  // SHA-256
    pub hash: String,
    pub generated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}
```

### 7.3 Chain of Custody

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustody {
    pub evidence_id: String,
    pub log: Vec<CustodyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEntry {
    pub sequence: u32,
    pub action: String,  // collection | analysis | transfer | storage
    pub timestamp: DateTime<Utc>,
    pub actor: Actor,
    pub hash_before: Option<String>,
    pub hash_after: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub role: String,
}
```

---

## 8. Report Generation

### 8.1 Markdown Report Structure

```rust
pub struct MarkdownReportGenerator {
    template_env: tera::Tera,
}

impl MarkdownReportGenerator {
    pub fn generate(&self, campaign: &CampaignResult) -> Result<String> {
        let mut context = Context::new();
        context.insert("campaign", &campaign.metadata);
        context.insert("summary", &campaign.summary());
        context.insert("waves", &campaign.waves);
        context.insert("findings", &campaign.findings_by_category());
        context.insert("llm_summary", &campaign.llm_analysis_summary());
        context.insert("evidence", &campaign.evidence_manifest());
        
        self.template_env.render("report.md.tera", &context)
            .map_err(|e| ReportError::TemplateError(e))
    }
}
```

### 8.2 Report Template (report.md.tera)

```tera
# GlassWorm Campaign Report: {{ campaign.name }}

**Case ID:** `{{ campaign.case_id }}`
**Date:** {{ campaign.started_at | date(format="%Y-%m-%d") }}
**Duration:** {{ campaign.duration }}

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Packages Scanned | {{ summary.total_packages }} |
| Malicious | {{ summary.malicious_packages }} |
| Suspicious | {{ summary.suspicious_packages }} |
| Clean | {{ summary.clean_packages }} |
| Detection Rate | {{ summary.detection_rate }}% |

{% if summary.malicious_packages > 0 %}
### Malicious Packages

| Package | Version | Threat Score | LLM Verdict | Category |
|---------|---------|--------------|-------------|----------|
{% for pkg in summary.malicious %}
| `{{ pkg.name }}` | {{ pkg.version }} | {{ pkg.threat_score }} | {{ pkg.llm_verdict }} | {{ pkg.category }} |
{% endfor %}
{% endif %}

---

## Wave Results

{% for wave in waves %}
### {{ wave.name }}

**Status:** {% if wave.status == "completed" %}✅ PASS{% elif wave.status == "failed" %}❌ FAIL{% else %}⚠️ {{ wave.status }}{% endif %}

{{ wave.description }}

**Packages:** {{ wave.packages_scanned }} scanned, {{ wave.packages_flagged }} flagged, {{ wave.packages_malicious }} malicious

{% endfor %}

---

## LLM Analysis Summary

### Tier 1 (Cerebras) Triage

- Packages analyzed: {{ llm_summary.tier1.analyzed }}
- Average analysis time: {{ llm_summary.tier1.avg_time }}s
- Model: {{ llm_summary.tier1.model }}

### Tier 2 (NVIDIA) Deep Analysis

- Packages analyzed: {{ llm_summary.tier2.analyzed }}
- Average analysis time: {{ llm_summary.tier2.avg_time }}s
- Models used:
{% for model in llm_summary.tier2.models_used %}
  - `{{ model.name }}`: {{ model.count }} packages
{% endfor %}

---

## Evidence Collected

| Package | Evidence Type | Path |
|---------|--------------|------|
{% for item in evidence.items %}
| `{{ item.source.name }}@{{ item.source.version }}` | {{ item.evidence_type }} | `{{ item.storage_path }}` |
{% endfor %}

---

## Appendix

### Scan Configuration

```toml
concurrency = {{ campaign.settings.concurrency }}
rate_limit = {{ campaign.settings.rate_limit_npm }}
llm_tier1 = {{ campaign.settings.llm.tier1_enabled }}
llm_tier2_threshold = {{ campaign.settings.llm.tier2_threshold }}
```

### Command Used

```bash
glassware-orchestrator campaign run {{ campaign.config_file }}
```
```

---

## 9. Implementation Phases

### Phase 1: Core Campaign System (Week 1)

| Task | Estimated Time | Dependencies |
|------|---------------|--------------|
| Campaign config parsing + validation | 1 day | - |
| DAG scheduler | 1 day | Config parsing |
| Wave executor (basic) | 2 days | DAG scheduler |
| Checkpoint manager (SQLite) | 2 days | Wave executor |
| Progress reporting (indicatif) | 1 day | Wave executor |

**Deliverable:** Basic campaign execution with checkpoint/resume

### Phase 2: Evidence & Reports (Week 2)

| Task | Estimated Time | Dependencies |
|------|---------------|--------------|
| Evidence collection system | 2 days | Wave executor |
| Chain of custody logging | 1 day | Evidence collection |
| Markdown report generation | 2 days | Evidence collection |
| SARIF report generation | 1 day | Evidence collection |

**Deliverable:** Full evidence collection and report generation

### Phase 3: Polish & Wave 6 (Week 3)

| Task | Estimated Time | Dependencies |
|------|---------------|--------------|
| Wave 6 campaign config | 0.5 days | - |
| Integration testing | 1.5 days | All Phase 1-2 |
| Documentation | 1 day | All features |
| Wave 6 execution | 1 day | Testing complete |

**Deliverable:** Wave 6 campaign completed with full reports

---

## 10. Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dag_scheduler_detects_circular_dependency() {
        // ...
    }
    
    #[test]
    fn test_checkpoint_save_and_resume() {
        // ...
    }
    
    #[test]
    fn test_evidence_integrity_verification() {
        // ...
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_wave_6_campaign_end_to_end() {
    // Run actual Wave 6 campaign with small sample
    // Verify:
    // - All waves execute in correct order
    // - Checkpoints saved correctly
    // - Evidence collected for flagged packages
    // - Reports generated successfully
}
```

---

## 11. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Circular dependencies in waves** | DAG validation at config load time |
| **Checkpoint corruption** | Transactional SQLite writes |
| **Rate limit exceeded** | Governor rate limiter with conservative defaults |
| **Evidence tampering** | SHA-256 hashes at every stage |
| **Long-running scan interruption** | Ctrl+C handler with graceful checkpoint save |
| **Memory exhaustion** | Streaming evidence writes, bounded worker pool |

---

## 12. Success Criteria

| Criterion | Target |
|-----------|--------|
| **Wave 6 completion** | 500+ packages scanned without data loss |
| **Checkpoint/resume** | Can resume from any interruption point |
| **Evidence integrity** | All evidence has valid SHA-256 hashes |
| **Report quality** | Markdown reports include all required sections |
| **Performance** | 500 packages in <1 hour with Tier 1 LLM |
| **Reliability** | Zero crashes during 24-hour test run |

---

**Next Steps:**
1. Review and approve this architecture
2. Begin Phase 1 implementation
3. Create Wave 6 campaign configuration
4. Execute Wave 6 campaign
