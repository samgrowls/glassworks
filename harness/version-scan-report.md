
# Version History Scan Report

**Generated:** 2026-03-21T06:48:20.539807
**Policy:** last-2
**Workers:** 2

## Summary

| Metric | Value |
|--------|-------|
| Total packages | 3 |
| Total versions scanned | 6 |
| Malicious versions | 0 |
| Malicious packages | 0 |
| Total findings | 0 |
| Failed scans | 6 |
| Duration | 2026-03-21T06:45:50.481332 to 2026-03-21T06:48:20.538327 |

## 🚨 Malicious Versions Detected

*No malicious versions detected*


## Database Location

Results saved to: `/tmp/glassware-tests/bg-test-results.db`

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
