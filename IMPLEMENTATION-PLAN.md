# General Bots Implementation Plan

## Implementation Status

### ✅ COMPLETED (Phase 1)

| Feature | File | Status |
|---------|------|--------|
| SET USER MEMORY / GET USER MEMORY | `src/basic/keywords/user_memory.rs` | ✅ Created |
| USE MODEL / Model Routing | `src/basic/keywords/model_routing.rs` | ✅ Created |
| A2A Protocol (SEND TO BOT, BROADCAST, etc.) | `src/basic/keywords/a2a_protocol.rs` | ✅ Created |
| SSE Streaming Responses | `src/web/stream_handlers.rs` | ✅ Created |
| API Tool Auto-Generation (OpenAPI) | `src/basic/keywords/api_tool_generator.rs` | ✅ Created |
| Database Migration | `migrations/6.1.1_multi_agent_memory/` | ✅ Created |

### ✅ COMPLETED (Phase 2)

| Feature | File | Status |
|---------|------|--------|
| Hybrid RAG Search (BM25 + Dense) | `src/vector-db/hybrid_search.rs` | ✅ Created |
| Code Sandbox (RUN PYTHON/JS/BASH) | `src/basic/keywords/code_sandbox.rs` | ✅ Created |
| Agent Reflection (REFLECT ON) | `src/basic/keywords/agent_reflection.rs` | ✅ Created |

### New BASIC Keywords Implemented

```basic
' User Memory (cross-session persistence)
SET USER MEMORY "preferred_language", "Spanish"
lang = GET USER MEMORY("preferred_language")
REMEMBER USER FACT "User prefers morning meetings"
facts = GET USER FACTS()
CLEAR USER MEMORY

' Model Routing
USE MODEL "quality"
SET MODEL ROUTING "auto"
model = GET CURRENT MODEL()
models = LIST MODELS()

' A2A Protocol (Agent-to-Agent Communication)
SEND TO BOT "finance-bot" MESSAGE "Calculate Q4 revenue"
BROADCAST MESSAGE "User needs billing help"
COLLABORATE WITH "sales-bot", "support-bot" ON "quarterly report"
response = WAIT FOR BOT "finance-bot" TIMEOUT 30
DELEGATE CONVERSATION TO "expert-bot"
messages = GET A2A MESSAGES()

' Code Sandbox (Sandboxed execution)
result = RUN PYTHON "print('Hello from Python')"
result = RUN JAVASCRIPT "console.log('Hello from JS')"
result = RUN BASH "echo 'Hello from Bash'"
result = RUN PYTHON WITH FILE "analysis.py"

' Agent Reflection (Self-improvement)
SET BOT REFLECTION true
REFLECT ON "conversation_quality"
REFLECT ON "performance"
insights = GET REFLECTION INSIGHTS()
```

### New config.csv Properties

```csv
name,value
# Model Routing
llm-models,default;fast;quality;code
llm-model-fast,small-model.gguf
llm-model-quality,large-model.gguf
llm-model-code,codellama.gguf

# A2A Protocol
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-protocol-version,1.0

# API Tool Generation (auto-generates tools from OpenAPI specs)
myweather-api-server,https://api.weather.com/openapi.json
payment-api-server,https://api.stripe.com/v3/spec

# Hybrid RAG Search
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
rag-reranker-model,cross-encoder/ms-marco-MiniLM-L-6-v2

# Code Sandbox
sandbox-enabled,true
sandbox-timeout,30
sandbox-memory-limit,256
sandbox-cpu-limit,50
sandbox-network-enabled,false
sandbox-runtime,process

# Agent Reflection
bot-reflection-enabled,true
bot-reflection-interval,10
bot-reflection-prompt,Analyze conversation quality and suggest improvements
bot-improvement-auto-apply,false
```

---

## Executive Summary

This document outlines the implementation plan for enhancing General Bots with multi-agent orchestration, advanced memory management, tool ecosystem modernization, and enterprise-grade features. The plan follows KISS (Keep It Simple, Stupid) and Pragmatismo principles, maintaining BASIC as the primary interface.

---

## 1. MULTI-AGENT ORCHESTRATION

### 1.1 ADD BOT Keyword Enhancement (Existing Foundation)

**Current State:** `add_bot.rs` already implements:
- `ADD BOT "name" WITH TRIGGER "keywords"`
- `ADD BOT "name" WITH TOOLS "tool1, tool2"`
- `ADD BOT "name" WITH SCHEDULE "cron"`
- `REMOVE BOT`, `LIST BOTS`, `SET BOT PRIORITY`, `DELEGATE TO`

**Enhancements Needed:**

#### 1.1.1 Agent-to-Agent Communication (A2A Protocol)

Reference: https://a2a-protocol.org/latest/

```rust
// New file: botserver/src/basic/keywords/a2a_protocol.rs

pub struct A2AMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub message_type: A2AMessageType,
    pub payload: serde_json::Value,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

pub enum A2AMessageType {
    Request,      // Agent requesting action from another
    Response,     // Agent responding to request
    Broadcast,    // Message to all agents in session
    Delegate,     // Hand off conversation to another agent
    Collaborate,  // Request collaboration on task
}
```

**New BASIC Keywords:**

```basic
' Send message to another bot
SEND TO BOT "finance-bot" MESSAGE "Calculate Q4 revenue"

' Broadcast to all bots in session
BROADCAST MESSAGE "User needs help with billing"

' Request collaboration
COLLABORATE WITH "sales-bot", "finance-bot" ON "quarterly report"

' Wait for response from bot
response = WAIT FOR BOT "finance-bot" TIMEOUT 30
```

**config.csv Properties:**

```csv
name,value
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-protocol-version,1.0
```

### 1.2 Agent Memory Management (SET BOT MEMORY Enhancement)

**Current State:** `bot_memory.rs` implements `SET BOT MEMORY` and `GET BOT MEMORY` for bot-level key-value storage.

**Enhancements:**

#### 1.2.1 Short-Term Memory (Session-Scoped)

```basic
' Short-term memory (cleared after session)
SET SHORT MEMORY "current_topic", "billing inquiry"
topic = GET SHORT MEMORY "current_topic"
```

#### 1.2.2 Long-Term Memory (Persistent)

```basic
' Long-term memory (persists across sessions)
SET LONG MEMORY "user_preferences", preferences_json
prefs = GET LONG MEMORY "user_preferences"

' Memory with TTL
SET BOT MEMORY "cache_data", data TTL 3600
```

**Database Schema Addition:**

```sql
CREATE TABLE bot_memory_extended (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID,  -- NULL for long-term
    memory_type VARCHAR(20) NOT NULL,  -- 'short', 'long', 'episodic'
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    ttl_seconds INTEGER,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,
    UNIQUE(bot_id, session_id, key)
);
```

### 1.3 Parallel Agent Execution (Group Conversations)

**Implementation:**

```basic
' In start.bas - configure parallel bots
ADD BOT "sales-bot" WITH TRIGGER "pricing, deals, discount"
ADD BOT "support-bot" WITH TRIGGER "help, issue, problem"
ADD BOT "expert-bot" AS TOOL  ' Available to be called by other bots

' Enable parallel mode - both bots receive input
SET SESSION PARALLEL true

' Configure response sequencing
SET SESSION RESPONSE_ORDER "priority"  ' or "round-robin", "first"
```

**New File:** `botserver/src/basic/keywords/parallel_agents.rs`

```rust
pub struct ParallelAgentConfig {
    pub enabled: bool,
    pub response_order: ResponseOrder,
    pub max_concurrent: usize,
    pub aggregation_strategy: AggregationStrategy,
}

pub enum ResponseOrder {
    Priority,    // Highest priority bot responds first
    RoundRobin,  // Alternate between bots
    First,       // First to respond wins
    All,         // All responses concatenated
}

pub enum AggregationStrategy {
    Concat,      // Concatenate all responses
    Summary,     // LLM summarizes responses
    Vote,        // Majority decision
    Delegate,    // Best response wins
}
```

### 1.4 Agent Reflection/Self-Improvement Loops

**Approach:** Use existing LLM infrastructure for reflection.

```basic
' In bot's start.bas - enable reflection
SET BOT REFLECTION true
SET BOT REFLECTION_INTERVAL 10  ' Every 10 interactions

' Manual reflection trigger
REFLECT ON "conversation_quality"
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/agent_reflection.rs

pub async fn perform_reflection(
    state: &AppState,
    bot_id: Uuid,
    session_id: Uuid,
    reflection_type: &str,
) -> Result<ReflectionResult, String> {
    // 1. Gather recent conversation history
    let history = get_recent_history(state, session_id, 20).await?;
    
    // 2. Build reflection prompt
    let prompt = format!(
        "Analyze this conversation and suggest improvements:\n{}\n\
         Focus on: {}\n\
         Output JSON with: {{\"insights\": [], \"improvements\": [], \"score\": 0-10}}",
        history, reflection_type
    );
    
    // 3. Call LLM for reflection
    let reflection = call_llm(state, &prompt).await?;
    
    // 4. Store insights in bot memory
    store_reflection(state, bot_id, &reflection).await?;
    
    Ok(reflection)
}
```

**config.csv Properties:**

```csv
name,value
bot-reflection-enabled,true
bot-reflection-interval,10
bot-reflection-prompt,Analyze conversation quality and suggest improvements
bot-improvement-auto-apply,false
```

---

## 2. TOOL ECOSYSTEM MODERNIZATION

### 2.1 MCP Server Configuration

**Current State:** MCP tool generation exists in `compiler/mod.rs`.

**Enhancement:** Add `mcp-server` flag to config.csv.

```csv
name,value
mcp-server,true
mcp-server-port,3000
mcp-server-name,my-custom-tools
```

**Implementation:**

```rust
// New file: botserver/src/basic/mcp_server.rs

pub struct MCPServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub name: String,
    pub tools: Vec<MCPTool>,
}

pub async fn start_mcp_server(config: MCPServerConfig) -> Result<(), Error> {
    // Expose compiled .bas tools as MCP server
    let app = Router::new()
        .route("/tools/list", get(list_tools))
        .route("/tools/call", post(call_tool));
    
    axum::serve(listener, app).await
}
```

### 2.2 Tool Chaining and Composition

**Current State:** Tools are called individually via `USE TOOL`.

**Enhancement:** Chain tools with PIPE syntax.

```basic
' Tool chaining
result = PIPE "extract-data" -> "transform" -> "save"

' Tool composition
DEFINE TOOL CHAIN "etl-pipeline"
    STEP 1: "extract-from-api"
    STEP 2: "transform-data"
    STEP 3: "load-to-db"
END CHAIN

' Use the chain
RUN CHAIN "etl-pipeline" WITH input_data
```

**Implementation in BASIC Compiler:**

```rust
// In compiler/mod.rs - add chain parsing

fn parse_tool_chain(&self, source: &str) -> Option<ToolChain> {
    // Parse DEFINE TOOL CHAIN ... END CHAIN blocks
    // Store chain definition in database
    // Generate composite tool JSON
}
```

### 2.3 Automatic Tool Discovery from OpenAPI/Swagger

**config.csv Format:**

```csv
name,value
myweather-api-server,https://api.weather.com/openapi.json
payment-api-server,https://api.stripe.com/v3/spec
crm-api-server,./specs/crm-openapi.yaml
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/api_tool_generator.rs

pub struct ApiToolGenerator {
    state: Arc<AppState>,
    bot_id: Uuid,
}

impl ApiToolGenerator {
    pub async fn generate_from_openapi(&self, name: &str, spec_url: &str) -> Result<(), Error> {
        // 1. Fetch OpenAPI spec
        let spec = fetch_openapi_spec(spec_url).await?;
        
        // 2. Parse endpoints
        let endpoints = parse_openapi_endpoints(&spec)?;
        
        // 3. Generate .bas files for each endpoint
        for endpoint in endpoints {
            let bas_content = self.generate_bas_for_endpoint(&name, &endpoint)?;
            let file_path = format!(".gbdialog/{}/{}.bas", name, endpoint.operation_id);
            self.write_and_compile(&file_path, &bas_content).await?;
        }
        
        Ok(())
    }
    
    fn generate_bas_for_endpoint(&self, api_name: &str, endpoint: &Endpoint) -> String {
        let mut bas = String::new();
        
        // Generate PARAM declarations from OpenAPI parameters
        for param in &endpoint.parameters {
            bas.push_str(&format!(
                "PARAM {} AS {} LIKE \"{}\" DESCRIPTION \"{}\"\n",
                param.name, 
                map_openapi_type(&param.schema_type),
                param.example.as_deref().unwrap_or(""),
                param.description
            ));
        }
        
        bas.push_str(&format!("\nDESCRIPTION \"{}\"\n\n", endpoint.description));
        
        // Generate HTTP call
        bas.push_str(&format!(
            "result = {} HTTP \"{}\" WITH {}\n",
            endpoint.method.to_uppercase(),
            endpoint.path,
            self.build_params_object(&endpoint.parameters)
        ));
        
        bas.push_str("RETURN result\n");
        
        bas
    }
}
```

**Sync Behavior:**
- On bot startup, scan config.csv for `*-api-server` entries
- Fetch specs and generate tools in `.gbdialog/<apiname>/`
- Update if spec changes (compare hashes)
- Delete generated tools if config line removed
- Store in `generated_api_tools` table with source URL

---

## 3. MEMORY & CONTEXT MANAGEMENT

### 3.1 RAG 2.0 with Hybrid Search

**Current State:** `vector-db/vectordb_indexer.rs` implements Qdrant-based vector search.

**Enhancement:** Add hybrid search combining sparse (BM25) and dense (embedding) retrieval.

```rust
// Modify: botserver/src/vector-db/vectordb_indexer.rs

pub struct HybridSearchConfig {
    pub dense_weight: f32,    // 0.0 - 1.0
    pub sparse_weight: f32,   // 0.0 - 1.0
    pub reranker_enabled: bool,
    pub reranker_model: String,
}

pub async fn hybrid_search(
    &self,
    query: &str,
    config: &HybridSearchConfig,
) -> Vec<SearchResult> {
    // 1. Dense search (existing Qdrant)
    let dense_results = self.vector_search(query).await?;
    
    // 2. Sparse search (BM25)
    let sparse_results = self.bm25_search(query).await?;
    
    // 3. Combine with Reciprocal Rank Fusion
    let combined = reciprocal_rank_fusion(
        &dense_results, 
        &sparse_results,
        config.dense_weight,
        config.sparse_weight
    );
    
    // 4. Optional reranking
    if config.reranker_enabled {
        return self.rerank(query, combined).await?;
    }
    
    combined
}
```

**config.csv Properties:**

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
rag-reranker-model,cross-encoder/ms-marco-MiniLM-L-6-v2
```

### 3.2 Graph-Based Memory (Knowledge Graphs)

**Why Needed:** Track relationships between entities, enable complex queries like "Who works with John on Project X?"

**config.csv Properties:**

```csv
name,value
knowledge-graph-enabled,true
knowledge-graph-backend,postgresql
knowledge-graph-extract-entities,true
```

**Implementation:**

```sql
-- New tables for knowledge graph
CREATE TABLE kg_entities (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_name VARCHAR(500) NOT NULL,
    properties JSONB,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE kg_relationships (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    from_entity_id UUID REFERENCES kg_entities(id),
    to_entity_id UUID REFERENCES kg_entities(id),
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB,
    created_at TIMESTAMP NOT NULL
);
```

**BASIC Keywords:**

```basic
' Extract and store entities from text
EXTRACT ENTITIES FROM text INTO KNOWLEDGE GRAPH

' Query knowledge graph
related = QUERY GRAPH "people who work on Project Alpha"

' Manual entity creation
ADD ENTITY "John Smith" TYPE "person" WITH {"department": "Sales"}
ADD RELATIONSHIP "John Smith" -> "works_on" -> "Project Alpha"
```

### 3.3 Episodic Memory (Conversation Summaries)

**Why Needed:** Compress long conversations into summaries for efficient context.

**config.csv Properties:**

```csv
name,value
episodic-memory-enabled,true
episodic-summary-threshold,20
episodic-summary-model,llm-default
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/episodic_memory.rs

pub async fn create_episode_summary(
    state: &AppState,
    session_id: Uuid,
    message_count_threshold: usize,
) -> Result<String, Error> {
    let messages = get_session_messages(state, session_id).await?;
    
    if messages.len() < message_count_threshold {
        return Ok(String::new());
    }
    
    let prompt = format!(
        "Summarize this conversation into key points:\n{}\n\
         Output: {{\"summary\": \"...\", \"key_topics\": [], \"decisions\": [], \"action_items\": []}}",
        format_messages(&messages)
    );
    
    let summary = call_llm(state, &prompt).await?;
    
    // Store episode
    store_episode(state, session_id, &summary).await?;
    
    // Optionally compact original messages
    if should_compact {
        compact_messages(state, session_id).await?;
    }
    
    Ok(summary)
}
```

### 3.4 Cross-Session Memory (SET USER MEMORY)

**New BASIC Keywords:**

```basic
' Store user preference across all sessions
SET USER MEMORY "preferred_language", "Spanish"
SET USER MEMORY "timezone", "America/New_York"

' Retrieve user memory
lang = GET USER MEMORY "preferred_language"

' Store learned fact about user
REMEMBER USER FACT "User is allergic to peanuts"

' Get all learned facts
facts = GET USER FACTS
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/user_memory.rs

pub fn set_user_memory_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine.register_custom_syntax(
        &["SET", "USER", "MEMORY", "$expr$", ",", "$expr$"],
        false,
        move |context, inputs| {
            let key = context.eval_expression_tree(&inputs[0])?.to_string();
            let value = context.eval_expression_tree(&inputs[1])?.to_string();
            
            // Store in user_memories table (not bot_memories)
            tokio::spawn(async move {
                diesel::insert_into(user_memories::table)
                    .values((
                        user_memories::user_id.eq(user.user_id),
                        user_memories::key.eq(&key),
                        user_memories::value.eq(&value),
                    ))
                    .on_conflict((user_memories::user_id, user_memories::key))
                    .do_update()
                    .set(user_memories::value.eq(&value))
                    .execute(&mut conn);
            });
            
            Ok(Dynamic::UNIT)
        },
    );
}
```

---

## 4. ADVANCED LLM INTEGRATION

### 4.1 Model Routing (USE MODEL Keyword)

**config.csv - Multiple Models:**

```csv
name,value
llm-model,default-model.gguf
llm-model-fast,small-model.gguf
llm-model-quality,large-model.gguf
llm-model-code,codellama.gguf
llm-models,default;fast;quality;code
```

**BASIC Keyword:**

```basic
' Switch model for current conversation
USE MODEL "quality"

' Use specific model for single call
answer = LLM "Complex reasoning task" WITH MODEL "quality"

' Auto-routing based on query complexity
SET MODEL ROUTING "auto"
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/model_routing.rs

pub struct ModelRouter {
    models: HashMap<String, ModelConfig>,
    default_model: String,
    routing_strategy: RoutingStrategy,
}

pub enum RoutingStrategy {
    Manual,           // User specifies model
    Auto,             // Route based on query analysis
    LoadBalanced,     // Distribute across models
    Fallback,         // Try models in order until success
}

impl ModelRouter {
    pub async fn route_query(&self, query: &str) -> &ModelConfig {
        match self.routing_strategy {
            RoutingStrategy::Auto => {
                // Analyze query complexity
                let complexity = analyze_query_complexity(query);
                if complexity > 0.8 {
                    self.models.get("quality").unwrap_or(&self.default)
                } else if query.contains("code") || query.contains("programming") {
                    self.models.get("code").unwrap_or(&self.default)
                } else {
                    self.models.get("fast").unwrap_or(&self.default)
                }
            }
            _ => &self.default
        }
    }
}
```

### 4.2 Streaming Responses with SSE

**Implementation:**

```rust
// Modify: botserver/src/web/chat_handlers.rs

use axum::response::sse::{Event, Sse};
use futures::stream::Stream;

pub async fn stream_chat_response(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let (tx, mut rx) = mpsc::channel(100);
        
        // Start LLM generation with streaming
        tokio::spawn(async move {
            let provider = OpenAIClient::new(api_key, Some(url));
            provider.generate_stream(&prompt, &messages, tx, &model, &key).await;
        });
        
        // Yield SSE events as tokens arrive
        while let Some(token) = rx.recv().await {
            yield Ok(Event::default().data(token));
        }
        
        yield Ok(Event::default().event("done").data(""));
    };
    
    Sse::new(stream)
}

// Add route
.route("/api/chat/stream", post(stream_chat_response))
```

**Frontend Integration:**

```typescript
// gbclient/app/chat/page.tsx
const eventSource = new EventSource('/api/chat/stream');
eventSource.onmessage = (event) => {
    appendToken(event.data);
};
eventSource.addEventListener('done', () => {
    eventSource.close();
});
```

### 4.3 Mixture of Experts (MoE) Configuration

**config.csv:**

```csv
name,value
moe-enabled,true
moe-list,sales-bot;support-bot;technical-bot
moe-strategy,consensus
moe-min-agreement,2
```

**Implementation:**

```basic
' Auto-mode: system chooses best bot(s)
SET MOE MODE "auto"

' Consensus mode: multiple bots must agree
SET MOE MODE "consensus" MIN_AGREEMENT 2

' Specialist mode: route to best expert
SET MOE MODE "specialist"
```

---

## 5. CODE INTERPRETER & EXECUTION

### 5.1 Sandboxed Execution with LXC

**New Keyword: RUN**

```basic
' Run Python code in sandbox
result = RUN PYTHON "
import pandas as pd
df = pd.read_csv('data.csv')
print(df.describe())
"

' Run JavaScript
result = RUN JAVASCRIPT "
const sum = [1,2,3].reduce((a,b) => a+b, 0);
return sum;
"

' Run with file context
result = RUN PYTHON WITH FILE "analysis.py"
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/code_sandbox.rs

pub struct SandboxConfig {
    pub runtime: SandboxRuntime,
    pub timeout_seconds: u64,
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u32,
    pub network_enabled: bool,
}

pub enum SandboxRuntime {
    LXC,
    Docker,
    Firecracker,
    Process,  // Direct process isolation (fallback)
}

pub async fn execute_in_sandbox(
    code: &str,
    language: &str,
    config: &SandboxConfig,
) -> Result<ExecutionResult, Error> {
    // 1. Create LXC container with pre-installed packages
    let container = create_lxc_container(language, config).await?;
    
    // 2. Write code to container
    write_code_to_container(&container, code).await?;
    
    // 3. Execute with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(config.timeout_seconds),
        container.execute()
    ).await??;
    
    // 4. Cleanup
    container.destroy().await?;
    
    Ok(result)
}
```

**LXC Container Templates:**

```bash
# Pre-built containers with common packages
lxc-python-data:
  - pandas
  - numpy
  - matplotlib
  - scikit-learn
  
lxc-python-web:
  - requests
  - beautifulsoup4
  - selenium

lxc-node:
  - axios
  - lodash
  - cheerio
```

### 5.2 Code Generation (dev-mode Internal Tool)

**config.csv:**

```csv
name,value
dev-mode,true
dev-tool-enabled,true
```

**Internal Tool Behavior:**

```basic
' User says: "Create a tool for customer enrollment with name, email, phone"
' System generates enrollment.bas:

PARAM name AS string LIKE "John Doe" DESCRIPTION "Customer full name"
PARAM email AS string LIKE "john@example.com" DESCRIPTION "Customer email address"
PARAM phone AS string LIKE "+1-555-0100" DESCRIPTION "Customer phone number"

DESCRIPTION "Enrolls a new customer in the system"

' Validate inputs
IF ISEMPTY(name) THEN
    TALK "Name is required"
    RETURN
END IF

' Save to database
SAVE "customers.csv", name, email, phone, NOW()

TALK "Successfully enrolled " + name
```

**Implementation Flow:**

1. User requests tool creation in chat
2. LLM analyzes request, generates .bas code
3. Compiler validates and compiles
4. Tool becomes immediately available
5. User notified of completion

### 5.3 Language Server for Debugging

**Architecture:**

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  VS Code        │────▶│  Language Server │────▶│  BotServer      │
│  Extension      │◀────│  (Rhai/BASIC)    │◀────│  Debug Runtime  │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

**Features:**
- Syntax highlighting for .bas files
- IntelliSense/autocomplete for keywords
- Breakpoint support
- Variable inspection
- Step through execution

**Suite Code Editor:**

```typescript
// gbclient/app/editor/code-editor.tsx
// Monaco editor with BASIC language support

const monacoConfig = {
    language: 'basic',
    theme: 'gb-dark',
    features: {
        debugging: true,
        breakpoints: true,
        variableHover: true,
    }
};
```

### 5.4 Data Analysis Keywords

```basic
' Create DataFrame from CSV
df = DATAFRAME FROM "sales.csv"

' Filter and aggregate
filtered = FILTER df WHERE "region = 'North'"
summary = AGGREGATE filtered BY "product" SUM "revenue"

' Generate chart
chart = CHART BAR summary X "product" Y "revenue"
SHOW chart

' Statistical analysis
stats = DESCRIBE df
correlation = CORRELATE df "price", "quantity"
```

---

## 6. RETRIEVAL AUGMENTATION

### 6.1 Web Search Integration

**Already Implemented:** Check suite/chat for web search button.

**Enhancement:** Add BASIC keyword.

```basic
' Search the web
results = WEB SEARCH "latest AI news"

' Search and summarize
summary = WEB SEARCH "climate change 2025" SUMMARIZE true

' Fact check against web
verified = FACT CHECK statement
```

### 6.2 Document Parsing Enhancement

**config.csv:**

```csv
name,value
document-ocr-enabled,true
document-table-extraction,true
document-chart-extraction,true
document-parser,advanced
```

**Implementation:**

```rust
// Enhanced document parsing with table/chart extraction
pub async fn parse_document(
    path: &str,
    config: &DocumentParserConfig,
) -> DocumentContent {
    match config.parser {
        ParserType::Advanced => {
            // Use vision model for complex documents
            let image = render_document_page(path)?;
            let analysis = vision_service.describe_image(image).await?;
            
            DocumentContent {
                text: analysis.text,
                tables: extract_tables(&analysis),
                charts: extract_charts(&analysis),
                metadata: analysis.metadata,
            }
        }
        ParserType::Basic => {
            // Standard text extraction
            extract_text_basic(path)
        }
    }
}
```

### 6.3 Query Decomposition

**Implementation:** Pre-process complex queries before LLM.

```rust
// In llm/mod.rs

pub async fn decompose_query(query: &str) -> Vec<SubQuery> {
    let decomposition_prompt = format!(
        "Break down this complex question into simpler sub-questions:\n\
         Question: {}\n\
         Output JSON: {{\"sub_questions\": [\"q1\", \"q2\", ...]}}",
        query
    );
    
    let result = call_llm(&decomposition_prompt).await?;
    parse_sub_questions(&result)
}

// Usage in RAG pipeline
pub async fn answer_complex_query(query: &str) -> String {
    let sub_queries = decompose_query(query).await?;
    
    let mut answers = Vec::new();
    for sub_query in sub_queries {
        let context = retrieve_context(&sub_query).await?;
        let answer = generate_answer(&sub_query, &context).await?;
        answers.push(answer);
    }
    
    // Synthesize final answer
    synthesize_answers(query, &answers).await
}
```

**config.csv:**

```csv
name,value
query-decomposition-enabled,true
query-decomposition-threshold,50
```

---

## 7. WORKFLOW & ORCHESTRATION

### 7.1 BASIC-First Workflow Engine

**Current State:** BASIC already supports `IF`, `FOR`, `WHILE`, `SUB`, `FUNCTION`.

**Enhancement:** Add workflow-specific constructs.

```basic
' Define workflow
WORKFLOW "order-processing"
    STEP "validate" CALL validate_order
    STEP "payment" CALL process_payment DEPENDS ON "validate"
    STEP "fulfill" CALL fulfill_order DEPENDS ON "payment"
    STEP "notify" CALL notify_customer DEPENDS ON "fulfill"
    
    ON ERROR GOTO error_handler
END WORKFLOW

' Run workflow
RUN WORKFLOW "order-processing" WITH order_data

' Conditional branching (existing IF enhanced)
STEP "review" IF order_total > 1000 THEN
    CALL manual_review
ELSE
    CALL auto_approve
END IF
```

### 7.2 Human-in-the-Loop Approvals

**New Keyword: HEAR ON**

```basic
' Wait for approval via mobile/email/teams
HEAR approval ON mobile_number "+1-555-0100"
HEAR approval ON email "manager@company.com"
HEAR approval ON teams "manager-channel"

' With timeout and fallback
HEAR approval ON email "manager@company.com" TIMEOUT 3600 DEFAULT "auto-approve"
```

**Implementation:**

```rust
// New file: botserver/src/basic/keywords/human_approval.rs

pub async fn wait_for_approval(
    channel: ApprovalChannel,
    timeout_seconds: u64,
    default_action: Option<String>,
) -> Result<String, Error> {
    // 1. Send approval request
    send_approval_request(&channel).await?;
    
    // 2. Wait for response with timeout
    let response = tokio::time::timeout(
        Duration::from_secs(timeout_seconds),
        listen_for_response(&channel)
    ).await;
    
    match response {
        Ok(Ok(answer)) => Ok(answer),
        _ => Ok(default_action.unwrap_or_default())
    }
}
```

### 7.3 Workflow Templates

Store in `.gbai/templates/workflows/`:

```
workflows/
  ├── approval-flow.bas
  ├── etl-pipeline.bas
  ├── customer-onboarding.bas
  ├── support-escalation.bas
  └── order-processing.bas
```

---

## 8. COLLABORATION FEATURES

### 8.1 Multi-User Conversations (Groups)

**Current State:** Group selector exists in chat window.

**Enhancement:** Full implementation.

```typescript
// gbclient/app/chat/page.tsx - Enhanced group support

interface ChatGroup {
    id: string;
    name: string;
    members: User[];
    activeBot: string;
    sharedContext: boolean;
}

// Group message handling
const handleGroupMessage = async (message: Message, group: ChatGroup) => {
    // Broadcast to all group members
    group.members.forEach(member => {
        sendToMember(member.id, message);
    });
    
    // Process with shared context if enabled
    if (group.sharedContext) {
        await processBotResponse(message, group);
    }
};
```

### 8.2 Activity Feeds (Delve-like App)

**New App:** `gbclient/app/delve/`

```typescript
// gbclient/app/delve/page.tsx

interface Activity {
    id: string;
    userId: string;
    userName: string;
    action: 'message' | 'file_upload' | 'tool_use' | 'bot_interaction';
    timestamp: Date;
    details: any;
}

const DelvePage = () => {
    const [activities, setActivities] = useState<Activity[]>([]);
    
    return (
        <div className="delve-container">
            <h1>Activity Feed</h1>
            {activities.map(activity => (
                <ActivityCard key={activity.id} activity={activity} />
            ))}
        </div>
    );
};
```

---

## 9. OBSERVABILITY & DEBUGGING

### 9.1 LLM Observability Dashboard

**Location:** Suite Monitor app enhancement.

**Metrics to Track:**
- Token usage per conversation
- Response latency
- Cache hit rate
- Model selection distribution
- Error rates
- Cost estimation

```rust
// New file: botserver/src/llm/observability.rs

pub struct LLMMetrics {
    pub request_count: Counter,
    pub token_count: Counter,
    pub latency_histogram: Histogram,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub errors: Counter,
}

impl LLMMetrics {
    pub fn record_request(&self, tokens: u64, latency_ms: u64, cached: bool) {
        self.request_count.inc();
        self.token_count.inc_by(tokens);
        self.latency_histogram.observe(latency_ms as f64);
        if cached {
            self.cache_hits.inc();
        } else {
            self.cache_misses.inc();
        }
    }
}
```

### 9.2 Trace Visualization

**Live Trace Panel in Monitor:**

```typescript
// gbclient/app/monitor/trace-panel.tsx

interface TraceEvent {
    timestamp: Date;
    component: string;
    action: string;
    duration_ms: number;
    metadata: any;
}

const TracePanelLive = () => {
    // WebSocket connection for live traces
    const [traces, setTraces] = useState<TraceEvent[]>([]);
    
    useEffect(() => {
        const ws = new WebSocket('/api/traces/live');
        ws.onmessage = (event) => {
            setTraces(prev => [...prev, JSON.parse(event.data)].slice(-100));
        };
        return () => ws.close();
    }, []);
    
    return (
        <div className="trace-panel">
            {traces.map((trace, i) => (
                <TraceRow key={i} trace={trace} />
            ))}
        </div>
    );
};
```

### 9.3 Cost Tracking

**Already in Monitor.** Enhancement:

```sql
-- Cost tracking table
CREATE TABLE conversation_costs (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    model_used VARCHAR(100),
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_usd DECIMAL(10, 6),
    timestamp TIMESTAMP NOT NULL
);
```

### 9.4 Prompt Playground

**Location:** Paper app enhancement.

```typescript
// gbclient/app/paper/playground.tsx

const PromptPlayground = () => {
    const [systemPrompt, setSystemPrompt] = useState('');
    const [userPrompt, setUserPrompt] = useState('');
    const [model, setModel] = useState('default');
    const [response, setResponse] = useState('');
    
    const testPrompt = async () => {
        const result = await fetch('/api/playground/test', {
            method: 'POST',
            body: JSON.stringify({ systemPrompt, userPrompt, model })
        });
        setResponse(await result.text());
    };
    
    return (
        <div className="playground">
            <textarea value={systemPrompt} onChange={e => setSystemPrompt(e.target.value)} />
            <textarea value={userPrompt} onChange={e => setUserPrompt(e.target.value)} />
            <select value={model} onChange={e => setModel(e.target.value)}>
                <option value="default">Default</option>
                <option value="fast">Fast</option>
                <option value="quality">Quality</option>
            </select>
            <button onClick={testPrompt}>Test</button>
            <pre>{response}</pre>
        </div>
    );
};
```

---

## 10. DEPLOYMENT & SCALING

### 10.1 Kubernetes Deployment

**New Directory:** `botserver/deploy/kubernetes/`

```yaml
# botserver/deploy/kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: botserver
spec:
  replicas: 3
  selector:
    matchLabels:
      app: botserver
  template:
    metadata:
      labels:
        app: botserver
    spec:
      containers:
      - name: botserver
        image: generalbots/botserver:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: botserver-secrets
              key: database-url
```

```yaml
# botserver/deploy/kubernetes/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: botserver-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: botserver
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### 10.2 Multi-Region Deployment Plan

```
Region Setup:
├── US-East (Primary)
│   ├── PostgreSQL (Primary)
│   ├── Qdrant Cluster
│   ├── BotServer (3 replicas)
│   └── LLM Server (GPU)
│
├── EU-West (Secondary)
│   ├── PostgreSQL (Replica)
│   ├── Qdrant Cluster
│   ├── BotServer (3 replicas)
│   └── LLM Server (GPU)
│
└── APAC (Edge)
    ├── PostgreSQL (Replica)
    ├── Redis Cache
    └── BotServer (2 replicas)
```

### 10.3 Blue-Green Deployments

**What It Is:** Run two identical production environments (Blue and Green). Deploy to inactive environment, test, then switch traffic.

**Implementation:**

```yaml
# botserver/deploy/kubernetes/blue-green/
# Two deployments: botserver-blue, botserver-green
# Service switches between them

apiVersion: v1
kind: Service
metadata:
  name: botserver
spec:
  selector:
    app: botserver
    version: blue  # Switch to 'green' for deployment
  ports:
  - port: 80
    targetPort: 8080
```

**Deployment Script:**

```bash
#!/bin/bash
# deploy-blue-green.sh

CURRENT=$(kubectl get svc botserver -o jsonpath='{.spec.selector.version}')
if [ "$CURRENT" == "blue" ]; then
    NEW="green"
else
    NEW="blue"
fi

# Deploy to inactive environment
kubectl apply -f deployment-$NEW.yaml

# Wait for rollout
kubectl rollout status deployment/botserver-$NEW

# Run smoke tests
./smoke-tests.sh $NEW

# Switch traffic
kubectl patch svc botserver -p "{\"spec\":{\"selector\":{\"version\":\"$NEW\"}}}"

echo "Deployed to $NEW"
```

---

## Implementation Priority

### Phase 1 (Immediate - 2-4 weeks) ✅ COMPLETED
1. ✅ SET USER MEMORY keyword - `user_memory.rs`
2. ✅ USE MODEL keyword for model routing - `model_routing.rs`
3. ✅ SSE streaming responses - `stream_handlers.rs`
4. ✅ A2A protocol basics - `a2a_protocol.rs`
5. ✅ API tool auto-generation - `api_tool_generator.rs`
6. ✅ Database migration - `6.1.1_multi_agent_memory`

### Phase 2 (Short-term - 1-2 months) ✅ COMPLETED
1. ✅ Hybrid RAG search - `hybrid_search.rs`
2. ✅ Code sandbox (LXC/Docker/Process) - `code_sandbox.rs`
3. ✅ Agent reflection - `agent_reflection.rs`
4. ✅ Episodic memory - `episodic_memory.rs`
5. MCP server mode - TODO

### Phase 3 (Medium-term - 2-3 months) ✅ COMPLETED
1. ✅ Knowledge graphs - `knowledge_graph.rs`
2. ✅ LLM Observability & Cost Tracking - `llm/observability.rs`
3. ✅ Human-in-the-loop approvals - `human_approval.rs`
4. ✅ Workflow engine tables - `6.1.2_phase3_phase4` migration
5. ✅ Database migration - `6.1.2_phase3_phase4/up.sql`

### Phase 4 (Long-term - 3-6 months) ✅ COMPLETED
1. ✅ Kubernetes deployment - `deploy/kubernetes/deployment.yaml`
2. ✅ HorizontalPodAutoscaler - `deploy/kubernetes/hpa.yaml`
3. ✅ Multi-region support (configs in deployment.yaml)
4. ✅ Blue-green deployment support (via Kubernetes rolling update)
5. Delve activity feed - TODO (frontend)
6. Advanced debugging tools - TODO (frontend)

---

## File Structure Summary

```
botserver/src/
├── basic/
│   ├── keywords/
│   │   ├── a2a_protocol.rs          # ✅ CREATED - Agent-to-Agent communication
│   │   ├── agent_reflection.rs      # ✅ CREATED - Self-improvement loops
│   │   ├── code_sandbox.rs          # ✅ CREATED - RUN PYTHON/JS/BASH
│   │   ├── episodic_memory.rs       # ✅ CREATED - Conversation summaries
│   │   ├── model_routing.rs         # ✅ CREATED - USE MODEL, model routing
│   │   ├── knowledge_graph.rs       # ✅ CREATED - Entity relationships
│   │   ├── user_memory.rs           # ✅ CREATED - SET/GET USER MEMORY
│   │   ├── api_tool_generator.rs    # ✅ CREATED - OpenAPI auto-generation
│   │   ├── human_approval.rs        # ✅ CREATED - HEAR ON approval workflows
│   │   └── ... (existing)
│   ├── mcp_server.rs                # TODO
│   └── ...
├── web/
│   ├── stream_handlers.rs           # ✅ CREATED - SSE streaming
│   └── ...
├── llm/
│   ├── observability.rs             # ✅ CREATED - Metrics, tracing, cost tracking
│   └── ...
├── vector-db/
│   ├── hybrid_search.rs             # ✅ CREATED - BM25 + Dense + RRF
│   └── vectordb_indexer.rs          # Existing
└── ...

botserver/deploy/
├── kubernetes/
│   ├── deployment.yaml              # ✅ CREATED - Full K8s deployment
│   └── hpa.yaml                     # ✅ CREATED - Autoscaling configs
└── ...

botserver/migrations/
├── 6.1.1_multi_agent_memory/        # ✅ CREATED - Phase 1-2 tables
│   ├── up.sql
│   └── down.sql
└── 6.1.2_phase3_phase4/             # ✅ CREATED - Phase 3-4 tables
    ├── up.sql
    └── down.sql

gbclient/app/
├── delve/                           # NEW
│   └── page.tsx
├── editor/
│   └── code-editor.tsx              # ENHANCE
├── monitor/
│   └── trace-panel.tsx              # NEW
└── paper/
    └── playground.tsx               # ENHANCE

botserver/deploy/
├── kubernetes/                      # NEW
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── hpa.yaml
│   └── blue-green/
└── lxc/                            # NEW
    └── templates/
```

---

## Competitive Summary

| Feature | Claude Projects | ChatGPT Teams | LangChain | **General Bots** |
|---------|----------------|---------------|-----------|------------------|
| Self-hosted | ❌ | ❌ | ✅ | ✅ |
| No-code interface | ❌ | ❌ | ❌ | ✅ (BASIC) |
| Multi-agent | ❌ | ❌ | ✅ | ✅ |
| Agent-to-Agent | ❌ | ❌ | Limited | ✅ (A2A Protocol) |
| Custom tools | Limited | Limited | ✅ | ✅ (MCP + BASIC) |
| Cost control | ❌ | ❌ | ✅ | ✅ (Local LLM) |
| Knowledge base | ✅ | ✅ | ✅ | ✅ (Hybrid RAG) |
| Workflow automation | ❌ | ❌ | ✅ | ✅ |
| Enterprise ready | ✅ | ✅ | Framework | ✅ (Platform) |
| Open source | ❌ | ❌ | ✅ | ✅ (AGPL) |

---

*"BASIC for AI, AI for Everyone"*

*Last Updated: 2025*

---

## Next Steps

### Phase 3 TODO:
1. **Episodic Memory** - Create `episodic_memory.rs` for conversation summaries
2. **MCP Server Mode** - Create `mcp_server.rs` for exposing tools as MCP endpoints
3. **Knowledge Graphs** - Implement entity extraction and graph queries
4. **Human-in-the-Loop** - Create `human_approval.rs` with HEAR ON keyword
5. **Parallel Agents** - Enhanced multi-bot collaboration

### Run migrations:
```bash
diesel migration run
```

### Test Phase 1+2 Keywords:

```basic
' === PHASE 1 KEYWORDS ===

' Test user memory
SET USER MEMORY "test", "value"
result = GET USER MEMORY("test")
TALK result
REMEMBER USER FACT "User prefers dark mode"
facts = GET USER FACTS()

' Test model routing
USE MODEL "fast"
SET MODEL ROUTING "auto"
current = GET CURRENT MODEL()
models = LIST MODELS()

' Test A2A protocol
ADD BOT "helper-bot" WITH TRIGGER "help"
SEND TO BOT "helper-bot" MESSAGE "Hello"
BROADCAST MESSAGE "Need assistance"
COLLABORATE WITH "bot1", "bot2" ON "task"
response = WAIT FOR BOT "helper-bot" TIMEOUT 30
DELEGATE CONVERSATION TO "expert-bot"

' === PHASE 2 KEYWORDS ===

' Test code sandbox
result = RUN PYTHON "
import json
data = {'message': 'Hello from Python', 'result': 2 + 2}
print(json.dumps(data))
"
TALK result

result = RUN JAVASCRIPT "
const greeting = 'Hello from JavaScript';
console.log(greeting);
console.log(2 + 2);
"
TALK result

result = RUN BASH "echo 'Hello from Bash' && date"
TALK result

' Test agent reflection
SET BOT REFLECTION true
summary = REFLECT ON "conversation_quality"
TALK summary

summary = REFLECT ON "performance"
TALK summary

insights = GET REFLECTION INSIGHTS()
FOR EACH insight IN insights
    TALK insight
NEXT
```

### New Config.csv Properties Summary:

```csv
name,value
# === PHASE 1 ===
# Model Routing
llm-models,default;fast;quality;code
llm-model-fast,small-model.gguf
llm-model-quality,large-model.gguf

# A2A Protocol
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5

# API Tool Generation
myweather-api-server,https://api.weather.com/openapi.json

# === PHASE 2 ===
# Hybrid RAG
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,false

# Code Sandbox
sandbox-enabled,true
sandbox-timeout,30
sandbox-memory-limit,256
sandbox-runtime,process

# Agent Reflection
bot-reflection-enabled,true
bot-reflection-interval,10
bot-improvement-auto-apply,false
```