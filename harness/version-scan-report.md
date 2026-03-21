
# Version History Scan Report

**Generated:** 2026-03-21T06:40:30.957460
**Policy:** last-3
**Workers:** 2

## Summary

| Metric | Value |
|--------|-------|
| Total packages | 10 |
| Total versions scanned | 30 |
| Malicious versions | 0 |
| Malicious packages | 0 |
| Total findings | 0 |
| Failed scans | 30 |
| Duration | 2026-03-21T06:40:24.193521 to 2026-03-21T06:40:30.956459 |

## 🚨 Malicious Versions Detected

*No malicious versions detected*


## Database Location

Results saved to: `/tmp/test-results.db`

## Query Examples

```sql
-- Find all malicious versions
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC;

-- Findings by package
SELECT package_name, SUM(findings_count) as total_findings
FROM version_scans
GROUP BY package_name
ORDER BY total_findings DESC;

-- Scan timeline
SELECT date(scan_timestamp) as date, COUNT(*) as versions_scanned
FROM version_scans
GROUP BY date
ORDER BY date;
```
