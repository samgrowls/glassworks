#!/bin/bash
# Evasion Hunter Scan - Target attacker evasion patterns

echo "======================================================================"
echo "EVASION HUNTER SCAN"
echo "======================================================================"
echo ""
echo "Targeting attacker evasion patterns:"
echo "  - Tier 1: Typosquatting (500 packages)"
echo "  - Tier 2: Org impersonation (500 packages)"  
echo "  - Tier 3: Fork bombs (300 packages)"
echo "  - Tier 4: GitHub abandoned repos (1000 repos)"
echo "  - Tier 5: Recent updates (500 packages)"
echo "  - Total: ~2,800 targets"
echo ""
echo "Estimated time: 4-8 hours"
echo ""

# Create package lists
cat > evasion-t1-typosquats.txt << 'TYPOSQUATS'
lodahs
ladash
ldash
lodashjs
axi0s
axois
axiosjs
axos
expres
expresss
expressjs-official
reacct
reactjs-official
react-lib
momnet
momentjs-official
mment
undersocre
underscorejs
underscore-js
reques
requestjs
reqeust
aysnc
asyncjs
asyc
webpakc
webpackjs
webpak
bael
babeljs-official
babael
chalkjs
commanderjs
debugjs
moment-timezone-official
TYPOSQUATS

cat > evasion-t2-fake-orgs.txt << 'FAKEORGS'
@microsoft-utils/request
@microsoft-tools/express
@google-cloud-tools/storage
@google-utils/analytics
@aws-sdk-extras/s3
@aws-tools/lambda
@facebook-react/components
@facebook-utils/fbjs
@vue-official/core
@angular-tools/cli
@nodejs-utils/fs
@npm-official/cli
TYPOORGS

cat > evasion-t3-forks.txt << 'FORKS'
lodash-maintained
express-active-fork
axios-updated
react-community-fork
moment-maintained
request-alternative
webpack-community
babel-updated
underscore-active
async-maintained
chalk-active
commander-fork
debug-maintained
moment-timezone-updated
FORKS

echo "=== Tier 1: Typosquatting Scan ==="
python3 optimized_scanner.py evasion-t1-typosquats.txt \
  -w 10 \
  -e data/evidence/evasion-t1 \
  -o evasion-t1-results.json \
  -n evasion-typosquats \
  2>&1 | tee evasion-t1.log

echo ""
echo "=== Tier 2: Fake Orgs Scan ==="
python3 optimized_scanner.py evasion-t2-fake-orgs.txt \
  -w 10 \
  -e data/evidence/evasion-t2 \
  -o evasion-t2-results.json \
  -n evasion-fake-orgs \
  2>&1 | tee evasion-t2.log

echo ""
echo "=== Tier 3: Fork Bombs Scan ==="
python3 optimized_scanner.py evasion-t3-forks.txt \
  -w 10 \
  -e data/evidence/evasion-t3 \
  -o evasion-t3-results.json \
  -n evasion-forks \
  2>&1 | tee evasion-t3.log

echo ""
echo "=== Tier 4: GitHub Abandoned Repos ==="
python3 github_scanner.py \
  --queries \
    "pushed:<2024-01-01 stars:>500" \
    "abandoned maintained fork" \
    "unmaintained alternative" \
  --repos-per-query 100 \
  --max-repos 300 \
  --scanner ./glassware-scanner \
  --output evasion-github-abandoned.json \
  --clone-dir data/github-clones-abandoned \
  2>&1 | tee evasion-github.log

echo ""
echo "======================================================================"
echo "EVASION HUNTER COMPLETE"
echo "======================================================================"
echo ""
echo "Results:"
echo "  - Tier 1: evasion-t1-results.json"
echo "  - Tier 2: evasion-t2-results.json"
echo "  - Tier 3: evasion-t3-results.json"
echo "  - Tier 4: evasion-github-abandoned.json"
echo ""
echo "Summary:"
for f in evasion-t*-results.json; do
  echo -n "$f: "
  cat $f | jq -r '.flagged' 2>/dev/null || echo "N/A"
done

