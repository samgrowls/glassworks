# Agent-Targeted Packages for GlassWorm Detection

**Rationale:** GlassWorm attackers target packages used by AI agents because:
1. High trust - agents execute code automatically
2. Wide reach - popular frameworks used by thousands
3. Tool access - MCP servers have access to external tools
4. Execution environment - code interpreters run arbitrary code

## High-Priority Targets

### MCP Servers (Model Context Protocol)
```
@modelcontextprotocol/inspector
@modelcontextprotocol/sdk
@anthropic-ai/mcp-server
mcp-server
@e2b/code-interpreter
```

### LangChain Ecosystem
```
langchain
@langchain/core
@langchain/openai
@langchain/anthropic
@langchain/community
langchain-core
langchain-openai
langchain-community
```

### AI SDKs
```
openai
anthropic
ai
@ai-sdk/openai
@ai-sdk/anthropic
@ai-sdk/google
@vercel/ai
@vercel/ai-sdk
```

### Agent Frameworks
```
crewai
@crewai/crewai
autoagent
@autoagent/core
babyagi
agentops
@agentops/agentops
smolagents
@huggingface/smolagents
litellm
```

### Code Execution
```
e2b
@e2b/sdk
@e2b/code-interpreter
@e2b/files
sandpack
@codesandbox/sdk
```

### Vector Databases (Agent Memory)
```
pinecone-client
@pinecone-database/pinecone
chromadb
@chromadb/core
weaviate-client
@weaviate/client
```

## Scan Command

```bash
cd harness

# Create package list
cat > agent-targets.txt << 'EOF'
[All packages from above]
EOF

# Scan with version history
python3 background_scanner.py \
  --packages agent-targets.txt \
  --policy last-20 \
  --output agent-scan-results.db \
  --workers 10
```
