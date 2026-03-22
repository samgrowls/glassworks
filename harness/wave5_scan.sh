#!/bin/bash
# Wave 5 Scanner — 1000 Package GlassWorm Hunt
# 
# Comprehensive scan targeting known GlassWorm patterns
# Uses release binaries from ~/.local/bin
#
# LLM Strategy:
#   - Cerebras (fast): Runs during scan for triage (--llm)
#   - NVIDIA (deep): Run separately on flagged packages (--deep-llm)
#
# Usage: ./wave5_scan.sh [--llm] [--deep-llm PACKAGE...]

set -e

GLASSWARE="$HOME/.local/bin/glassware-orchestrator"
OUTPUT_DIR="$(dirname "$0")/data/wave5-results"
EVIDENCE_DIR="$HOME/glassworks-archive/evidence"
mkdir -p "$OUTPUT_DIR"

if [ ! -x "$GLASSWARE" ]; then
    echo "Error: glassware-orchestrator not found at $GLASSWARE"
    exit 1
fi

# Parse arguments
USE_LLM=""
DEEP_LLM_PKGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --llm)
            USE_LLM="--llm"
            # Set Cerebras environment for fast triage
            export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
            export GLASSWARE_LLM_API_KEY="${GLASSWARE_LLM_API_KEY:-}"
            export GLASSWARE_LLM_MODEL="llama-3.3-70b"
            echo "LLM triage enabled (Cerebras - fast)"
            shift
            ;;
        --deep-llm)
            shift
            while [[ $# -gt 0 && $1 != --* ]]; do
                DEEP_LLM_PKGS+=("$1")
                shift
            done
            echo "Deep LLM analysis enabled (NVIDIA) for: ${DEEP_LLM_PKGS[*]}"
            ;;
        *)
            shift
            ;;
    esac
done

echo "============================================================"
echo "Wave 5 Scanner — 1000 Package GlassWorm Hunt"
echo "============================================================"
echo ""
echo "Binary: $GLASSWARE"
echo "Output: $OUTPUT_DIR"
echo "LLM: ${USE_LLM:-disabled}"
echo ""

TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Evidence tarballs
EVIDENCE_TARBALLS=()
for f in "$EVIDENCE_DIR"/*.tgz; do
    [ -f "$f" ] && EVIDENCE_TARBALLS+=("$f")
done

# Wave 5A: React Native Ecosystem (200 packages) - HIGH PRIORITY
REACT_NATIVE=(
    "react-native-country-picker@1.0.2" "react-native-phone-input@1.3.7"
    "react-native-phone-number-input@2.1.0" "react-native-otp-inputs@0.3.1"
    "react-native-sms-retriever@1.0.2" "react-native-international-phone-number@0.11.4"
    "react-native-locale@0.0.15" "react-native-localize@3.0.6"
    "react-native-i18n@2.0.15" "react-native-globalize@1.0.0"
    "react-native-translation@1.0.0" "react-native-languages@3.0.0"
    "react-native-country-code-picker@1.0.0" "react-native-flags@1.0.0"
    "react-native-picker@4.3.0" "react-native-modal@13.0.1"
    "react-native-dropdown-picker@5.4.6" "react-native-vector-icons@10.0.0"
    "react-native-gesture-handler@2.14.0" "react-native-reanimated@3.6.0"
    "react-native-safe-area-context@4.8.0" "react-native-screens@3.29.0"
    "react-native-navigation@7.39.0" "react-native-router-flux@4.3.1"
    "react-navigation@4.4.4" "@react-navigation/native@6.1.0"
    "react-native-maps@1.7.0" "react-native-webview@13.6.0"
    "react-native-image-picker@7.1.0" "react-native-camera@4.2.1"
    "react-native-vision-camera@3.6.0" "react-native-video@6.0.0"
    "react-native-audio@4.3.0" "react-native-sound@0.11.2"
    "react-native-fs@2.20.0" "react-native-share@10.0.0"
    "react-native-linkedin@2.0.0" "react-native-facebook-login@2.0.0"
    "react-native-google-signin@11.0.0" "react-native-apple-authentication@2.3.0"
    "react-native-biometrics@3.0.0" "react-native-touch-id@4.4.1"
    "react-native-fingerprint-scanner@6.0.0" "react-native-permissions@4.0.0"
    "react-native-device-info@10.12.0" "react-native-system-info@2.0.0"
    "react-native-network-logger@1.15.0" "react-native-flipper@0.226.0"
    "react-native-debugger@0.0.5" "react-native-devtools@1.0.0"
)

# Wave 5B: MCP/AI Infrastructure (150 packages)
MCP_AI=(
    "langchain@0.1.0" "openai@4.28.0" "@anthropic-ai/sdk@0.18.0"
    "ai@3.0.0" "replicate@0.27.0" "cohere-ai@7.9.0"
    "@langchain/core@0.1.0" "@langchain/openai@0.0.0"
    "llamaindex@0.1.0" "@e2b/code-interpreter@0.0.1"
    "mcp@1.0.0" "@modelcontextprotocol/sdk@0.1.0"
    "huggingface@0.0.1" "@huggingface/inference@2.6.0"
    "transformers@5.0.0" "tokenizers@0.15.0"
    "sentence-transformers@0.0.0" "langfuse@2.0.0"
    "langsmith@0.1.0" "dspy-ai@2.0.0"
    "guidance@0.1.0" "outlines@0.0.0"
    "vllm@0.0.0" "text-generation@0.7.0"
    "instructor@1.0.0" "pydantic-ai@0.0.0"
    "agno@0.0.0" "crewai@0.0.0"
    "autogen@0.0.0" "semantic-kernel@1.0.0"
    "auto-gpt@0.0.0" "babyagi@0.0.0"
    "gpt-index@0.0.0" "chromadb@0.0.0"
    "pinecone-client@0.0.0" "weaviate-client@0.0.0"
    "qdrant-client@0.0.0" "milvus@0.0.0"
)

# Wave 5C: Unicode/Locale Heavy (200 packages)
UNICODE_LOCALE=(
    "globalize@1.7.0" "cldrjs@0.5.5" "i18n-iso-countries@7.14.0"
    "i18n-js@4.5.0" "country-data@0.0.31" "timezone-js@0.4.13"
    "date-format@4.0.14" "node-gettext@3.0.0" "gettext-parser@8.0.0"
    "i18next@23.0.0" "react-i18next@14.0.0" "vue-i18n@9.0.0"
    "polyglot@0.4.3" "babelfish@1.0.0" "transliteration@2.3.0"
    "cldr-data@36.0.0" "cldr-core@36.0.0" "cldr-dates@36.0.0"
    "cldr-numbers@36.0.0" "cldr-units@36.0.0" "cldr-localenames@36.0.0"
    "country-list@2.3.0" "country-code-lookup@0.1.0"
    "iso-3166@0.1.0" "iso-countries@0.0.0"
    "world-countries@5.0.0" "i18n-node@0.0.0"
    "node-i18n@0.0.0" "express-i18n@0.0.0"
    "locale@0.0.15" "locales@0.0.0"
    "moment-timezone@0.5.0" "dayjs@1.11.0" "date-fns-tz@2.0.0"
    "luxon@3.0.0" "js-joda@0.0.0" "chrono-node@2.0.0"
    "rrule@2.0.0" "ical.js@0.0.0" "ical-generator@0.0.0"
    "node-ical@0.0.0" "ical-expander@0.0.0"
)

# Wave 5D: Install Scripts/Native (200 packages)
INSTALL_SCRIPTS=(
    "node-gyp@10.1.0" "bindings@1.5.0" "prebuild@11.0.0"
    "nan@2.18.0" "node-addon-api@7.1.0" "cmake-js@7.3.0"
    "node-pre-gyp@0.17.0" "prebuild-install@7.1.0"
    "node-sass@9.0.0" "sass@1.70.0" "chromedriver@122.0.0"
    "geckodriver@4.0.0" "sharp@0.33.0" "canvas@2.11.0"
    "sqlite3@5.1.0" "better-sqlite3@9.0.0" "leveldown@6.0.0"
    "rocksdb@0.0.0" "secp256k1@5.0.0" "tiny-secp256k1@0.0.0"
    "bigint-buffer@0.0.0" "bufferutil@0.0.0" "utf-8-validate@0.0.0"
    "argon2@0.0.0" "bcrypt@5.0.0" "scrypt@0.0.0"
    "keccak@0.0.0" "sha3@0.0.0" "crypto-js@0.0.0"
    "libpq@0.0.0" "pg-native@0.0.0" "mysql-native@0.0.0"
    "oracledb@0.0.0" "tedious@0.0.0" "mssql@0.0.0"
    "mongoose@0.0.0" "mongodb@0.0.0" "redis@0.0.0"
    "ioredis@0.0.0" "memcached@0.0.0" "cassandra-driver@0.0.0"
)

# Wave 5E: Random Recent/Popular (250 packages)
RANDOM_POPULAR=(
    "next@14.1.0" "nuxt@3.10.0" "svelte@4.2.0"
    "prisma@5.10.0" "tailwindcss@3.4.0" "zod@3.22.0"
    "valtio@1.13.0" "zustand@4.5.0" "immer@10.0.0"
    "jotai@2.6.0" "recoil@0.7.7" "mobx@6.12.0"
    "redux@5.0.0" "react-redux@9.0.0" "vuex@4.1.0"
    "pinia@2.1.0" "express@4.19.2" "fastify@4.26.0"
    "koa@2.15.0" "@hapi/hapi@21.0.0" "@nestjs/core@10.0.0"
    "axios@1.6.0" "got@14.0.0" "node-fetch@3.0.0"
    "undici@6.0.0" "superagent@9.0.0" "request@0.0.0"
    "lodash@4.17.0" "underscore@0.0.0" "ramda@0.0.0"
    "async@3.0.0" "bluebird@0.0.0" "q@0.0.0"
    "when@0.0.0" "p-queue@0.0.0" "p-limit@0.0.0"
    "promise@0.0.0" "native-promise@0.0.0"
    "chalk@5.0.0" "colors@0.0.0" "cli-color@0.0.0"
    "ora@0.0.0" "inquirer@0.0.0" "commander@0.0.0"
    "yargs@0.0.0" "minimist@0.0.0" "meow@0.0.0"
    "glob@0.0.0" "fast-glob@0.0.0" "rimraf@0.0.0"
    "del@0.0.0" "cpy@0.0.0" "fs-extra@0.0.0"
    "graceful-fs@0.0.0" "chokidar@0.0.0" "watchpack@0.0.0"
    "parcel@0.0.0" "rollup@0.0.0" "webpack@0.0.0"
    "vite@0.0.0" "esbuild@0.0.0" "swc@0.0.0"
    "babel@0.0.0" "typescript@0.0.0" "eslint@0.0.0"
    "prettier@0.0.0" "stylelint@0.0.0" "markdownlint@0.0.0"
    "jest@0.0.0" "mocha@0.0.0" "vitest@0.0.0"
    "cypress@0.0.0" "playwright@0.0.0" "puppeteer@0.0.0"
    "selenium@0.0.0" "webdriverio@0.0.0" "testcafe@0.0.0"
    "ava@0.0.0" "tape@0.0.0" "jasmine@0.0.0"
    "chai@0.0.0" "sinon@0.0.0" "nock@0.0.0"
    "supertest@0.0.0" "http-server@0.0.0" "serve@0.0.0"
    "live-server@0.0.0" "browser-sync@0.0.0" "webpack-dev-server@0.0.0"
    "nodemon@0.0.0" "pm2@0.0.0" "forever@0.0.0"
    "supervisor@0.0.0" "node-monit@0.0.0" "newrelic@0.0.0"
    "datadog@0.0.0" "sentry@0.0.0" "logrocket@0.0.0"
    "hotjar@0.0.0" "fullstory@0.0.0" "amplitude@0.0.0"
    "mixpanel@0.0.0" "segment@0.0.0" "analytics@0.0.0"
    "plausible@0.0.0" "umami@0.0.0" "fathom@0.0.0"
)

ALL_PACKAGES=(
    "${REACT_NATIVE[@]}"
    "${MCP_AI[@]}"
    "${UNICODE_LOCALE[@]}"
    "${INSTALL_SCRIPTS[@]}"
    "${RANDOM_POPULAR[@]}"
)

echo "Wave 5A: React Native Ecosystem (${#REACT_NATIVE[@]} packages)"
echo "Wave 5B: MCP/AI Infrastructure (${#MCP_AI[@]} packages)"
echo "Wave 5C: Unicode/Locale Heavy (${#UNICODE_LOCALE[@]} packages)"
echo "Wave 5D: Install Scripts/Native (${#INSTALL_SCRIPTS[@]} packages)"
echo "Wave 5E: Random Recent/Popular (${#RANDOM_POPULAR[@]} packages)"
echo ""
echo "Total: ${#ALL_PACKAGES[@]} npm packages + ${#EVIDENCE_TARBALLS[@]} evidence"
echo ""
echo "Starting scan... This will take 45-90 minutes..."
echo ""

# Step 1: Scan evidence
if [ ${#EVIDENCE_TARBALLS[@]} -gt 0 ]; then
    echo "Step 1: Scanning evidence tarballs..."
    TARBALL_OUTPUT="$OUTPUT_DIR/wave5-evidence-$TIMESTAMP.json"
    
    $GLASSWARE scan-tarball "${EVIDENCE_TARBALLS[@]}" $USE_LLM \
        --output "$TARBALL_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave5-evidence-log-$TIMESTAMP.txt"
    echo ""
fi

# Step 2: Scan npm packages
echo "Step 2: Scanning ${#ALL_PACKAGES[@]} npm packages..."
NPM_OUTPUT="$OUTPUT_DIR/wave5-npm-$TIMESTAMP.json"

$GLASSWARE scan-npm "${ALL_PACKAGES[@]}" $USE_LLM \
    --output "$NPM_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave5-npm-log-$TIMESTAMP.txt"

echo ""
echo "============================================================"
echo "WAVE 5 COMPLETE"
echo "============================================================"
echo ""
echo "Evidence results: $TARBALL_OUTPUT"
echo "NPM results: $NPM_OUTPUT"
echo ""

# Summary
if command -v jq &> /dev/null; then
    echo "=== SUMMARY ==="
    
    if [ -f "$TARBALL_OUTPUT" ]; then
        echo "Evidence:"
        jq -r '.summary | "  Packages: \(.total_packages)\n  Malicious: \(.malicious_packages)"' "$TARBALL_OUTPUT" 2>/dev/null || echo "  No summary"
    fi
    
    if [ -f "$NPM_OUTPUT" ]; then
        echo ""
        echo "NPM:"
        jq -r '.summary | "  Packages: \(.total_packages)\n  Malicious: \(.malicious_packages)\n  Avg Score: \(.average_threat_score)"' "$NPM_OUTPUT" 2>/dev/null || echo "  No summary"
        
        echo ""
        echo "Malicious packages:"
        jq -r '.results[] | select(.is_malicious == true) | "  - \(.package_name)@\(.version) (score: \(.threat_score))"' "$NPM_OUTPUT" 2>/dev/null | head -20
    fi
fi

echo ""
echo "Review results with:"
echo "  cat $NPM_OUTPUT | jq '.results[] | select(.is_malicious == true)'"

# Deep LLM Analysis (NVIDIA) - Run separately on flagged packages
if [ ${#DEEP_LLM_PKGS[@]} -gt 0 ]; then
    echo ""
    echo "============================================================"
    echo "Running Deep LLM Analysis (NVIDIA)"
    echo "============================================================"
    
    DEEP_OUTPUT="$OUTPUT_DIR/wave5-deep-llm-$TIMESTAMP.json"
    
    # Set NVIDIA environment
    export GLASSWARE_LLM_BASE_URL="https://integrate.api.nvidia.com/v1"
    export GLASSWARE_LLM_API_KEY="${NVIDIA_API_KEY:-}"
    export GLASSWARE_LLM_MODEL="qwen/qwen3.5-397b-a17b"
    
    echo "Analyzing ${#DEEP_LLM_PKGS[@]} flagged packages with NVIDIA..."
    $GLASSWARE scan-npm "${DEEP_LLM_PKGS[@]}" --llm \
        --output "$DEEP_OUTPUT" --format json 2>&1 | tee "$OUTPUT_DIR/wave5-deep-llm-log-$TIMESTAMP.txt"
    
    echo ""
    echo "Deep analysis results: $DEEP_OUTPUT"
    
    if [ -f "$DEEP_OUTPUT" ]; then
        echo ""
        echo "LLM Verdicts:"
        jq -r '.results[] | "\(.package_name): \(.llm_verdict // "No verdict")"' "$DEEP_OUTPUT" 2>/dev/null || echo "  No verdicts available"
    fi
fi
