# Multi-Agent Office Suite - Complete Design Document

## 🎯 Vision: Beat Microsoft 365, Google Workspace & All AI Competitors

**General Bots = Multi-Agent AI + Complete Office Suite + Research Engine + Banking + Everything**

This document outlines the complete implementation plan to make General Bots the world's most powerful FREE enterprise platform.

---

## 📋 Table of Contents

1. [BOT Keyword - Multi-Agent System](#1-bot-keyword---multi-agent-system)
2. [Chat UI Enhancements](#2-chat-ui-enhancements)
3. [Conversational Banking (bank.gbai)](#3-conversational-banking-bankgbai)
4. [Excel Clone (HTMX/Rust)](#4-excel-clone-htmxrust)
5. [Word Editor for .docx](#5-word-editor-for-docx)
6. [M365/Office Competitive Analysis](#6-m365office-competitive-analysis)
7. [Google/MS Graph API Compatibility](#7-googlems-graph-api-compatibility)
8. [Copilot/Gemini Feature Parity](#8-copilotgemini-feature-parity)
9. [Attachment System (Plus Button)](#9-attachment-system-plus-button)
10. [Conversation Branching](#10-conversation-branching)
11. [PLAY Keyword - Content Projector](#11-play-keyword---content-projector)
12. [Implementation Priority](#12-implementation-priority)

---

## 1. BOT Keyword - Multi-Agent System

### Concept

Every conversation becomes a **group conversation** where multiple specialized bots can participate. Bots join based on triggers (tools, schedules, keywords) and collaborate to answer complex queries.

### Keywords

```basic
' Add a bot to the conversation
ADD BOT "finance-expert" WITH TRIGGER "money, budget, invoice, payment"
ADD BOT "legal-advisor" WITH TRIGGER "contract, agreement, compliance"
ADD BOT "hr-assistant" WITH TRIGGER "employee, vacation, hiring"

' Add bot with tool-based trigger
ADD BOT "data-analyst" WITH TOOLS "AGGREGATE, CHART, REPORT"

' Add bot with schedule-based participation
ADD BOT "daily-reporter" WITH SCHEDULE "0 9 * * *"

' Remove bot from conversation
REMOVE BOT "finance-expert"

' List active bots
bots = LIST BOTS

' Set bot priority (who answers first)
SET BOT PRIORITY "legal-advisor", 1

' Bot-to-bot delegation
DELEGATE TO "specialist-bot" WITH CONTEXT current_conversation

' Create bot swarm for complex tasks
CREATE SWARM "research-team" WITH BOTS "researcher, analyst, writer"
```

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    CONVERSATION ORCHESTRATOR                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User Message ──▶ Trigger Analyzer ──▶ Bot Selector             │
│                         │                    │                   │
│                         ▼                    ▼                   │
│              ┌─────────────────┐    ┌──────────────┐            │
│              │ Keyword Triggers │    │ Tool Triggers │           │
│              │ - finance terms  │    │ - AGGREGATE   │           │
│              │ - legal terms    │    │ - CHART       │           │
│              │ - hr terms       │    │ - specific    │           │
│              └─────────────────┘    └──────────────┘            │
│                         │                    │                   │
│                         ▼                    ▼                   │
│              ┌─────────────────────────────────────┐            │
│              │         BOT RESPONSE AGGREGATOR      │            │
│              │  - Merge responses                   │            │
│              │  - Resolve conflicts                 │            │
│              │  - Format for user                   │            │
│              └─────────────────────────────────────┘            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Database Schema

```sql
-- Bot definitions
CREATE TABLE bots (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    system_prompt TEXT,
    model_config JSONB,
    tools JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bot triggers
CREATE TABLE bot_triggers (
    id UUID PRIMARY KEY,
    bot_id UUID REFERENCES bots(id),
    trigger_type VARCHAR(50), -- 'keyword', 'tool', 'schedule', 'event'
    trigger_config JSONB,
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true
);

-- Session bot associations
CREATE TABLE session_bots (
    id UUID PRIMARY KEY,
    session_id UUID,
    bot_id UUID REFERENCES bots(id),
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true
);

-- Bot message history
CREATE TABLE bot_messages (
    id UUID PRIMARY KEY,
    session_id UUID,
    bot_id UUID REFERENCES bots(id),
    content TEXT,
    role VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Rust Implementation

```rust
// src/basic/keywords/add_bot.rs

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTrigger {
    pub trigger_type: TriggerType,
    pub keywords: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Keyword,
    Tool,
    Schedule,
    Event,
}

pub fn add_bot_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    // ADD BOT "name" WITH TRIGGER "keywords"
    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "TRIGGER", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context.eval_expression_tree(&inputs[0])?.to_string();
            let trigger = context.eval_expression_tree(&inputs[1])?.to_string();
            
            let state_for_thread = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    add_bot_to_session(
                        &state_for_thread,
                        session_id,
                        &bot_name,
                        BotTrigger {
                            trigger_type: TriggerType::Keyword,
                            keywords: Some(trigger.split(',').map(|s| s.trim().to_string()).collect()),
                            tools: None,
                            schedule: None,
                        }
                    ).await
                });
                let _ = tx.send(result);
            });
            
            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "ADD BOT timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    );

    // ADD BOT "name" WITH TOOLS "tool1, tool2"
    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "TOOLS", "$expr$"],
        false,
        move |context, inputs| {
            // Similar implementation for tool-based triggers
        },
    );

    // ADD BOT "name" WITH SCHEDULE "cron"
    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "SCHEDULE", "$expr$"],
        false,
        move |context, inputs| {
            // Similar implementation for schedule-based triggers
        },
    );
}

async fn add_bot_to_session(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
    trigger: BotTrigger,
) -> Result<String, String> {
    // Implementation to add bot to session
}
```

### Multi-Agent Orchestrator

```rust
// src/core/multi_agent.rs

use std::collections::HashMap;
use uuid::Uuid;

pub struct MultiAgentOrchestrator {
    state: Arc<AppState>,
    active_bots: HashMap<Uuid, BotInstance>,
}

impl MultiAgentOrchestrator {
    pub async fn process_message(
        &self,
        session_id: Uuid,
        message: &str,
    ) -> Result<Vec<BotResponse>, Error> {
        // 1. Get all active bots for this session
        let bots = self.get_session_bots(session_id).await?;
        
        // 2. Analyze message and match triggers
        let matching_bots = self.match_triggers(&bots, message).await?;
        
        // 3. If no specific bot matches, use default
        if matching_bots.is_empty() {
            return self.default_bot_response(session_id, message).await;
        }
        
        // 4. Get responses from all matching bots
        let mut responses = Vec::new();
        for bot in matching_bots {
            let response = self.get_bot_response(&bot, session_id, message).await?;
            responses.push(response);
        }
        
        // 5. Aggregate responses
        let final_response = self.aggregate_responses(responses).await?;
        
        Ok(final_response)
    }

    async fn match_triggers(
        &self,
        bots: &[BotInstance],
        message: &str,
    ) -> Vec<BotInstance> {
        let mut matching = Vec::new();
        let message_lower = message.to_lowercase();
        
        for bot in bots {
            if let Some(trigger) = &bot.trigger {
                match trigger.trigger_type {
                    TriggerType::Keyword => {
                        if let Some(keywords) = &trigger.keywords {
                            for keyword in keywords {
                                if message_lower.contains(&keyword.to_lowercase()) {
                                    matching.push(bot.clone());
                                    break;
                                }
                            }
                        }
                    }
                    TriggerType::Tool => {
                        // Check if message implies using specific tools
                    }
                    _ => {}
                }
            }
        }
        
        // Sort by priority
        matching.sort_by(|a, b| b.priority.cmp(&a.priority));
        matching
    }

    async fn aggregate_responses(
        &self,
        responses: Vec<BotResponse>,
    ) -> Result<Vec<BotResponse>, Error> {
        // Use LLM to merge multiple bot responses into coherent answer
        // Or return all responses with bot attribution
        Ok(responses)
    }
}
```

---

## 2. Chat UI Enhancements

### 2.1 Poe/Perplexity-Style Features

#### Chat Interface Components

```html
<!-- botserver/ui/suite/chat/enhanced-chat.html -->

<div class="chat-container" id="chat-app" hx-ext="ws" ws-connect="/ws">
    <!-- Bot Selector Bar (Poe-style) -->
    <div class="bot-selector-bar">
        <div class="active-bots" 
             hx-get="/api/chat/active-bots"
             hx-trigger="load, bot-changed from:body"
             hx-swap="innerHTML">
            <!-- Active bots appear here -->
        </div>
        <button class="add-bot-btn" 
                hx-get="/api/bots/available"
                hx-target="#bot-modal"
                hx-swap="innerHTML">
            + Add Bot
        </button>
    </div>

    <!-- Connection Status -->
    <div id="connection-status" class="connection-status">
        <span class="status-dot"></span>
        <span class="status-text">Connected</span>
    </div>

    <!-- Messages with Bot Attribution -->
    <main id="messages" class="messages-container">
        <!-- Messages load here with bot avatars and names -->
    </main>

    <!-- Typing Indicators for Multiple Bots -->
    <div id="typing-indicators" class="typing-indicators hidden">
        <!-- Shows which bots are "thinking" -->
    </div>

    <!-- Enhanced Input Area -->
    <footer class="input-footer">
        <!-- Suggestions -->
        <div class="suggestions-container" id="suggestions"
             hx-get="/api/suggestions"
             hx-trigger="load"
             hx-swap="innerHTML">
        </div>

        <!-- Attachment Preview -->
        <div id="attachment-preview" class="attachment-preview hidden">
            <!-- Previews of attached files -->
        </div>

        <!-- Input Form -->
        <form class="input-container"
              hx-post="/api/chat/send"
              hx-target="#messages"
              hx-swap="beforeend"
              hx-encoding="multipart/form-data"
              hx-on::after-request="this.reset(); clearAttachments();">
            
            <!-- Plus Button for Attachments -->
            <div class="attachment-menu">
                <button type="button" class="plus-btn" onclick="toggleAttachmentMenu()">
                    <span>+</span>
                </button>
                <div id="attachment-dropdown" class="attachment-dropdown hidden">
                    <button type="button" onclick="attachImage()">
                        📷 Image
                    </button>
                    <button type="button" onclick="attachDocument()">
                        📄 Document
                    </button>
                    <button type="button" onclick="attachAudio()">
                        🎵 Audio
                    </button>
                    <button type="button" onclick="attachVideo()">
                        🎬 Video
                    </button>
                    <button type="button" onclick="attachCode()">
                        💻 Code
                    </button>
                    <button type="button" onclick="useCamera()">
                        📸 Camera
                    </button>
                    <button type="button" onclick="useScreenshot()">
                        🖥️ Screenshot
                    </button>
                </div>
            </div>

            <!-- Hidden file inputs -->
            <input type="file" id="image-input" accept="image/*" multiple hidden>
            <input type="file" id="document-input" accept=".pdf,.doc,.docx,.xls,.xlsx,.ppt,.pptx,.txt,.csv" multiple hidden>
            <input type="file" id="audio-input" accept="audio/*" hidden>
            <input type="file" id="video-input" accept="video/*" hidden>
            <input type="file" id="code-input" accept=".js,.ts,.py,.rs,.go,.java,.c,.cpp,.h,.css,.html,.json,.yaml,.xml,.sql,.sh,.bas" hidden>

            <!-- Message Input -->
            <textarea
                name="content"
                id="message-input"
                placeholder="Message... (@ to mention a bot)"
                rows="1"
                autofocus
                required
            ></textarea>

            <!-- Voice Button -->
            <button type="button" id="voice-btn" title="Voice Input"
                    hx-post="/api/voice/start"
                    hx-swap="none">
                🎤
            </button>

            <!-- Send Button -->
            <button type="submit" id="send-btn" title="Send">
                ↑
            </button>
        </form>
    </footer>

    <!-- Branch Indicator -->
    <div id="branch-indicator" class="branch-indicator hidden">
        <span>Branch from message #<span id="branch-from"></span></span>
        <button onclick="cancelBranch()">Cancel</button>
    </div>

    <!-- Scroll to Bottom -->
    <button class="scroll-to-bottom hidden" id="scroll-to-bottom">↓</button>

    <!-- Projector/Player Modal -->
    <div id="projector-modal" class="projector-modal hidden">
        <div class="projector-header">
            <span id="projector-title">Content Viewer</span>
            <button onclick="closeProjector()">✕</button>
        </div>
        <div id="projector-content" class="projector-content">
            <!-- Content plays here -->
        </div>
        <div class="projector-controls">
            <button onclick="projectorPrev()">◀</button>
            <button onclick="projectorPlayPause()">⏯</button>
            <button onclick="projectorNext()">▶</button>
            <button onclick="projectorFullscreen()">⛶</button>
        </div>
    </div>
</div>
```

### 2.2 Simple Chat/Talk UIs

#### Intercom-Style Widget

```html
<!-- botserver/ui/widgets/intercom.html -->

<div class="intercom-widget" id="intercom-widget">
    <button class="intercom-trigger" onclick="toggleIntercom()">
        <span class="intercom-icon">💬</span>
        <span class="intercom-badge" id="unread-count">0</span>
    </button>
    
    <div class="intercom-panel hidden" id="intercom-panel">
        <div class="intercom-header">
            <img src="/static/bot-avatar.png" class="bot-avatar">
            <div class="bot-info">
                <span class="bot-name">Assistant</span>
                <span class="bot-status">Online</span>
            </div>
            <button onclick="closeIntercom()">✕</button>
        </div>
        
        <div class="intercom-messages" id="intercom-messages"
             hx-get="/api/chat/messages"
             hx-trigger="load"
             hx-swap="innerHTML">
        </div>
        
        <form class="intercom-input"
              hx-post="/api/chat/send"
              hx-target="#intercom-messages"
              hx-swap="beforeend">
            <input type="text" name="content" placeholder="Type a message...">
            <button type="submit">Send</button>
        </form>
    </div>
</div>

<style>
.intercom-widget {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 9999;
}

.intercom-trigger {
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    position: relative;
}

.intercom-badge {
    position: absolute;
    top: -5px;
    right: -5px;
    background: #ff4444;
    color: white;
    border-radius: 50%;
    width: 20px;
    height: 20px;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.intercom-panel {
    position: absolute;
    bottom: 70px;
    right: 0;
    width: 350px;
    height: 500px;
    background: white;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.15);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.intercom-header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 16px;
    display: flex;
    align-items: center;
    gap: 12px;
}

.intercom-messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
}

.intercom-input {
    padding: 12px;
    border-top: 1px solid #eee;
    display: flex;
    gap: 8px;
}

.intercom-input input {
    flex: 1;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 20px;
    outline: none;
}

.intercom-input button {
    padding: 10px 20px;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 20px;
    cursor: pointer;
}
</style>
```

#### PTT (Push-to-Talk) Interface

```html
<!-- botserver/ui/widgets/ptt.html -->

<div class="ptt-interface" id="ptt-interface">
    <div class="ptt-status" id="ptt-status">
        <span class="status-icon">🔇</span>
        <span class="status-text">Press and hold to talk</span>
    </div>
    
    <div class="ptt-visualizer" id="ptt-visualizer">
        <!-- Audio waveform visualization -->
        <canvas id="waveform-canvas"></canvas>
    </div>
    
    <button class="ptt-button" 
            id="ptt-button"
            onmousedown="startRecording()"
            onmouseup="stopRecording()"
            ontouchstart="startRecording()"
            ontouchend="stopRecording()">
        <span class="ptt-icon">🎤</span>
        <span class="ptt-label">PUSH TO TALK</span>
    </button>
    
    <div class="ptt-response" id="ptt-response">
        <!-- Bot response plays here -->
    </div>
    
    <div class="ptt-history" id="ptt-history">
        <!-- Conversation history -->
    </div>
</div>

<style>
.ptt-interface {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(180deg, #1a1a2e 0%, #16213e 100%);
    color: white;
    padding: 20px;
}

.ptt-button {
    width: 150px;
    height: 150px;
    border-radius: 50%;
    background: linear-gradient(145deg, #e74c3c 0%, #c0392b 100%);
    border: 4px solid #fff;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    font-size: 40px;
    box-shadow: 0 8px 24px rgba(231, 76, 60, 0.4);
    transition: all 0.2s ease;
    user-select: none;
}

.ptt-button:active {
    transform: scale(0.95);
    background: linear-gradient(145deg, #27ae60 0%, #1e8449 100%);
    box-shadow: 0 4px 16px rgba(39, 174, 96, 0.6);
}

.ptt-button.recording {
    animation: pulse 1s infinite;
}

@keyframes pulse {
    0% { box-shadow: 0 0 0 0 rgba(39, 174, 96, 0.7); }
    70% { box-shadow: 0 0 0 30px rgba(39, 174, 96, 0); }
    100% { box-shadow: 0 0 0 0 rgba(39, 174, 96, 0); }
}

.ptt-visualizer {
    width: 100%;
    max-width: 300px;
    height: 100px;
    margin: 20px 0;
}

.ptt-status {
    margin-bottom: 20px;
    font-size: 18px;
    display: flex;
    align-items: center;
    gap: 10px;
}
</style>

<script>
let mediaRecorder;
let audioChunks = [];

async function startRecording() {
    const button = document.getElementById('ptt-button');
    button.classList.add('recording');
    
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    mediaRecorder = new MediaRecorder(stream);
    
    mediaRecorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
    };
    
    mediaRecorder.start();
    document.getElementById('ptt-status').innerHTML = 
        '<span class="status-icon">🔴</span><span class="status-text">Recording...</span>';
}

async function stopRecording() {
    const button = document.getElementById('ptt-button');
    button.classList.remove('recording');
    
    mediaRecorder.stop();
    
    mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: 'audio/webm' });
        audioChunks = [];
        
        // Send to server for transcription and response
        const formData = new FormData();
        formData.append('audio', audioBlob);
        
        document.getElementById('ptt-status').innerHTML = 
            '<span class="status-icon">⏳</span><span class="status-text">Processing...</span>';
        
        const response = await fetch('/api/voice/ptt', {
            method: 'POST',
            body: formData
        });
        
        const result = await response.json();
        
        // Play response audio
        if (result.audio_url) {
            const audio = new Audio(result.audio_url);
            audio.play();
        }
        
        document.getElementById('ptt-status').innerHTML = 
            '<span class="status-icon">🔇</span><span class="status-text">Press and hold to talk</span>';
    };
}
</script>
```

#### Totem/Kiosk Interface

```html
<!-- botserver/ui/widgets/totem.html -->

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bot Totem</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #0f0f23 0%, #1a1a3e 100%);
            color: white;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }
        
        .totem-header {
            padding: 30px;
            text-align: center;
            background: rgba(255,255,255,0.05);
        }
        
        .totem-logo {
            font-size: 48px;
            margin-bottom: 10px;
        }
        
        .totem-title {
            font-size: 24px;
            font-weight: 300;
        }
        
        .totem-main {
            flex: 1;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 40px;
        }
        
        .avatar-container {
            width: 200px;
            height: 200px;
            border-radius: 50%;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 80px;
            margin-bottom: 40px;
            box-shadow: 0 0 60px rgba(102, 126, 234, 0.5);
            animation: breathe 3s infinite ease-in-out;
        }
        
        @keyframes breathe {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.05); }
        }
        
        .avatar-container.listening {
            animation: listening 0.5s infinite ease-in-out;
            box-shadow: 0 0 80px rgba(39, 174, 96, 0.8);
        }
        
        @keyframes listening {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.1); }
        }
        
        .message-display {
            text-align: center;
            font-size: 28px;
            max-width: 800px;
            line-height: 1.5;
            margin-bottom: 40px;
        }
        
        .quick-actions {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 20px;
            max-width: 600px;
        }
        
        .quick-action {
            padding: 30px 20px;
            background: rgba(255,255,255,0.1);
            border: 1px solid rgba(255,255,255,0.2);
            border-radius: 16px;
            cursor: pointer;
            text-align: center;
            transition: all 0.3s ease;
        }
        
        .quick-action:hover {
            background: rgba(255,255,255,0.2);
            transform: translateY(-5px);
        }
        
        .quick-action-icon {
            font-size: 40px;
            margin-bottom: 10px;
        }
        
        .quick-action-label {
            font-size: 16px;
        }
        
        .totem-footer {
            padding: 20px;
            text-align: center;
            background: rgba(0,0,0,0.3);
        }
        
        .touch-hint {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 10px;
            font-size: 18px;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <header class="totem-header">
        <div class="totem-logo">🤖</div>
        <h1 class="totem-title">How can I help you today?</h1>
    </header>
    
    <main class="totem-main">
        <div class="avatar-container" id="avatar">
            🤖
        </div>
        
        <div class="message-display" id="message">
            Touch any option below or tap the microphone to speak
        </div>
        
        <div class="quick-actions">
            <div class="quick-action" onclick="quickAction('directions')">
                <div class="quick-action-icon">🗺️</div>
                <div class="quick-action-label">Directions</div>
            </div>
            <div class="quick-action" onclick="quickAction('schedule')">
                <div class="quick-action-icon">📅</div>
                <div class="quick-action-label">Schedule</div>
            </div>
            <div class="quick-action" onclick="quickAction('services')">
                <div class="quick-action-icon">🏢</div>
                <div class="quick-action-label">Services</div>
            </div>
            <div class="quick-action" onclick="quickAction('contact')">
                <div class="quick-action-icon">📞</div>
                <div class="quick-action-label">Contact</div>
            </div>
            <div class="quick-action" onclick="startVoice()">
                <div class="quick-action-icon">🎤</div>
                <div class="quick-action-label">Speak</div>
            </div>
            <div class="quick-action" onclick="quickAction('help')">
                <div class="quick-action-icon">❓</div>
                <div class="quick-action-label">Help</div>
            </div>
        </div>
    </main>
    
    <footer class="totem-footer">
        <div class="touch-hint">
            <span>👆</span>
            <span>Touch to interact</span>
        </div>
    </footer>
    
    <script>
        async function quickAction(action) {
            document.getElementById('message').textContent = 'Processing...';
            
            const response = await fetch('/api/totem/action', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ action })
            });
            
            const result = await response.json();
            document.getElementById('message').textContent = result.message;
            
            if (result.audio_url) {
                const audio = new Audio(result.audio_url);
                audio.play();
            }
        }
        
        async function startVoice() {
            const avatar = document.getElementById('avatar');
            avatar.classList.add('listening');
            document.getElementById('message').textContent = 'Listening...';
            
            // Implement voice recognition
        }
    </script>
</body>
</html>
```

---

## 3. Conversational Banking (bank.gbai)

### Complete Banking Template

```
templates/bank.gbai/
├── bank.gbdialog/
│   └── start.json
├── bank.gbot/
│   └── config.csv
├── bank.gbkb/
│   └── banking-faq.md
├── dialogs/
│   ├── account.bas
│   ├── transfer.bas
│   ├── payment.bas
│   ├── loan.bas
│   ├── investment.bas
│   ├── cards.bas
│   └── support.bas
├── tables/
│   ├── accounts.csv
│   ├── transactions.csv
│   ├── cards.csv
│   ├── loans.csv
│   ├── beneficiaries.csv
│   └── scheduled_payments.csv
└── README.md
```

### Bank Configuration

```csv
# bank.gbot/config.csv
key,value
bank-name,General Bank
bank-code,001
swift-code,GENBBRSP
support-phone,0800-123-4567
support-email,support@generalbank.com
pix-enabled,true
ted-enabled,true
doc-enabled,true
boleto-enabled,true
credit-card-enabled,true
debit-card-enabled,true
investment-enabled,true
loan-enabled,true
insurance-enabled,true
two-factor-auth,true
transaction-limit-default,5000.00
daily-limit-default,20000.00
```

### Account Management

```basic
' dialogs/account.bas

' Show account balance
SUB ShowBalance()
    user_id = GET USER ID
    
    accounts = FIND "accounts.csv" WHERE user_id = user_id
    
    IF LEN(accounts) = 0 THEN
        TALK "You don't have any accounts registered. Would you like to open one?"
        RETURN
    END IF
    
    TALK "Here are your account balances:"
    TALK ""
    
    total = 0
    FOR EACH account IN accounts
        TALK "📊 **" + account.account_type + " Account**"
        TALK "   Account: " + account.account_number
        TALK "   Balance: R$ " + FORMAT(account.balance, "0.00")
        TALK "   Available: R$ " + FORMAT(account.available_balance, "0.00")
        TALK ""
        total = total + account.balance
    NEXT
    
    TALK "💰 **Total Balance: R$ " + FORMAT(total, "0.00") + "**"
END SUB

' Show recent transactions
SUB ShowTransactions(account_number, days)
    IF days = "" THEN days = 30 END IF
    
    start_date = DATEADD(NOW(), -days, "day")
    
    transactions = FIND "transactions.csv" WHERE account_number = account_number AND date >= start_date ORDER BY date DESC LIMIT 20
    
    IF LEN(transactions) = 0 THEN
        TALK "No transactions found in the last " + days + " days."
        RETURN
    END IF
    
    TALK "📋 **Recent Transactions**"
    TALK ""
    
    FOR EACH tx IN transactions
        IF tx.type = "credit" THEN
            icon = "💵"
            sign = "+"
        ELSE
            icon = "💸"
            sign = "-"
        END IF
        
        TALK icon + " " + FORMAT(tx.date, "dd/MM") + " | " + tx.description
        TALK "   " + sign + "R$ " + FORMAT(tx.amount, "0.00") + " | Balance: R$ " + FORMAT(tx.balance_after, "0.00")
        TALK ""
    NEXT
END SUB

' Generate account statement
SUB GenerateStatement(account_number, start_date, end_date)
    transactions = FIND "transactions.csv" WHERE account_number = account_number AND date >= start_date AND date <= end_date ORDER BY date
    
    TABLE statement
        COLUMN "Date" FORMAT "dd/MM/yyyy"
        COLUMN "Description"
        COLUMN "Type"
        COLUMN "Amount" FORMAT "R$ #,##0.00"
        COLUMN "Balance" FORMAT "R$ #,##0.00"
        
        FOR EACH tx IN transactions
            ROW tx.date, tx.description, tx.type, tx.amount, tx.balance_after
        NEXT
    END TABLE
    
    ' Export to PDF
    pdf_file = EXPORT TABLE statement TO "pdf" WITH TITLE "Account Statement - " + account_number
    
    TALK "Your statement is ready!"
    TALK "📄 [Download Statement](" + pdf_file + ")"
    
    ' Send by email
    email = GET USER email
    IF email <> "" THEN
        SEND MAIL email, "Your Account Statement", "Please find attached your account statement.", pdf_file
        TALK "I've also sent a copy to your email."
    END IF
END SUB

' Open new account
SUB OpenAccount(account_type)
    user_id = GET USER ID
    user = GET USER
    
    ' Verify KYC
    IF NOT user.kyc_verified THEN
        TALK "To open a new account, we need to verify your identity first."
        CALL VerifyKYC()
        RETURN
    END IF
    
    ' Generate account number
    account_number = GenerateAccountNumber()
    
    ' Create account
    TABLE new_account
        ROW account_number, user_id, account_type, 0.00, 0.00, NOW(), "active"
    END TABLE
    
    SAVE "accounts.csv", new_account
    
    TALK "🎉 Congratulations! Your " + account_type + " account has been created!"
    TALK ""
    TALK "📋 **Account Details**"
    TALK "Account Number: " + account_number
    TALK "Type: " + account_type
    TALK "Status: Active"
    TALK ""
    TALK "Your virtual debit card is being generated..."
    
    ' Create virtual card
    CALL CreateVirtualCard(account_number)
END SUB

FUNCTION GenerateAccountNumber()
    ' Generate unique account number
    branch = "0001"
    sequence = GET BOT MEMORY "account_sequence"
    IF sequence = "" THEN sequence = 10000 END IF
    sequence = sequence + 1
    SET BOT MEMORY "account_sequence", sequence
    
    account = branch + "-" + FORMAT(sequence, "000000")
    digit = CalculateCheckDigit(account)
    
    RETURN account + "-" + digit
END FUNCTION
```

### Money Transfers

```basic
' dialogs/transfer.bas

' PIX Transfer
SUB PIXTransfer()
    TALK "Let's make a PIX transfer. What type of key will you use?"
    
    ADD SUGGESTION "CPF/CNPJ"
    ADD SUGGESTION "Phone"
    ADD SUGGESTION "Email"
    ADD SUGGESTION "Random Key"
    
    key_type = HEAR
    
    TALK "Enter the PIX key:"
    pix_key = HEAR
    
    ' Validate and get recipient info
    recipient = ValidatePIXKey(key_type, pix_key)
    
    IF recipient.error THEN
        TALK "❌ Invalid PIX key. Please check and try again."
        RETURN
    END IF
    
    TALK "Recipient: **" + recipient.name + "**"
    TALK "Bank: " + recipient.bank_name
    TALK ""
    TALK "Enter the amount to transfer:"
    
    amount = HEAR
    amount = ParseMoney(amount)
    
    ' Check balance and limits
    account = GET USER primary_account
    
    IF amount > account.available_balance THEN
        TALK "❌ Insufficient balance. Available: R$ " + FORMAT(account.available_balance, "0.00")
        RETURN
    END IF
    
    daily_used = GetDailyTransferTotal(account.account_number)
    daily_limit = GET USER daily_transfer_limit
    
    IF daily_used + amount > daily_limit THEN
        TALK "❌ This transfer would exceed your daily limit."
        TALK "Daily limit: R$ " + FORMAT(daily_limit, "0.00")
        TALK "Already used: R$ " + FORMAT(daily_used, "0.00")
        TALK "Available: R$ " + FORMAT(daily_limit - daily_used, "0.00")
        RETURN
    END IF
    
    ' Confirm transaction
    TALK "📤 **Transfer Summary**"
    TALK "To: " + recipient.name
    TALK "PIX Key: " + MaskPIXKey(pix_key)
    TALK "Amount: R$ " + FORMAT(amount, "0.00")
    TALK ""
    TALK "Confirm this transfer?"
    
    ADD SUGGESTION "Yes, confirm"
    ADD SUGGESTION "No, cancel"
    
    confirmation = HEAR
    
    IF confirmation CONTAINS "yes" OR confirmation CONTAINS "confirm" THEN
        ' Request 2FA
        TALK "For your security, enter the code sent to your phone:"
        code = HEAR
        
        IF NOT Verify2FA(code) THEN
            TALK "❌ Invalid code. Transfer cancelled for security."
            RETURN
        END IF
        
        ' Execute transfer
        result = ExecutePIXTransfer(account.account_number, recipient, amount)
        
        IF result.success THEN
            TALK "✅ **Transfer completed!**"
            TALK "Transaction ID: " + result.transaction_id
            TALK "New balance: R$ " + FORMAT(result.new_balance, "0.00")
            
            ' Save transaction
            TABLE transaction
                ROW result.transaction_id, account.account_number, "pix_out", amount, result.new_balance, NOW(), recipient.pix_key, recipient.name, "completed"
            END TABLE
            SAVE "transactions.csv", transaction
        ELSE
            TALK "❌ Transfer failed: " + result.error
        END IF
    ELSE
        TALK "Transfer cancelled."
    END IF
END SUB

' TED Transfer
SUB TEDTransfer()
    TALK "Let's make a TED transfer."
    
    ' Get recipient bank info
    TALK "Enter the bank code (e.g., 001 for Banco do Brasil):"
    bank_code = HEAR
    
    TALK "Enter the branch number:"
    branch = HEAR
    
    TALK "Enter the account number (with digit):"
    account_number = HEAR
    
    TALK "Enter the recipient's full name:"
    recipient_name = HEAR
    
    TALK "Enter the recipient's CPF/CNPJ:"
    document = HEAR
    
    TALK "Enter the amount to transfer:"
    amount = HEAR
    amount = ParseMoney(amount)
    
    ' Validate and process similar to PIX
    ' ... (similar flow with bank validation)
END SUB

' Schedule recurring transfer
SUB ScheduleTransfer()
    TALK "Let's schedule a recurring transfer."
    
    TALK "How often should the transfer occur?"
    ADD SUGGESTION "Weekly"
    ADD SUGGESTION "Monthly"
    ADD SUGGESTION "Custom"
    
    frequency = HEAR
    
    ' Get transfer details
    TALK "Enter the PIX key of the recipient:"
    pix_key = HEAR
    
    TALK "Enter the amount:"
    amount = HEAR
    
    TALK "When should the first transfer occur?"
    start_date = HEAR
    
    ' Create scheduled payment
    TABLE scheduled
        ROW GenerateID(), GET USER ID, "pix", pix_key, amount, frequency, start_date, "active"
    END TABLE
    
    SAVE "scheduled_payments.csv", scheduled
    
    ' Set up the schedule
    SET SCHEDULE frequency WITH START start_date
        CALL ExecuteScheduledTransfer(scheduled.id)
    END SCHEDULE
    
    TALK "✅ Recurring transfer scheduled!"
    TALK "First transfer: " + FORMAT(start_date, "dd/MM/yyyy")
    TALK "Frequency: " + frequency
    TALK "Amount: R$ " + FORMAT(amount, "0.00")
END SUB
```

### Bill Payment

```basic
' dialogs/payment.bas

' Pay bill/boleto
SUB PayBoleto()
    TALK "Enter the barcode or paste the boleto line:"
    barcode = HEAR
    
    ' Parse boleto
    boleto = ParseBoleto(barcode)
    
    IF boleto.error THEN
        TALK "❌ Invalid barcode. Please check and try again."
        RETURN
    END IF
    
    TALK "📄 **Bill Details**"
    TALK "Beneficiary: " + boleto.beneficiary
    TALK "Amount: R$ " + FORMAT(boleto.amount, "0.00")
    TALK "Due date: " + FORMAT(boleto.due_date, "dd/MM/yyyy")
    
    IF boleto.is_overdue THEN
        TALK "⚠️ This bill is overdue. Late fees may apply."
        TALK "Original amount: R$ " + FORMAT(boleto.original_amount, "0.00")
        TALK "Late fee: R$ " + FORMAT(boleto.late_fee, "0.00")
        TALK "Interest: R$ " + FORMAT(boleto.interest, "0.00")
    END IF
    
    TALK ""
    TALK "Pay this bill?"
    
    ADD SUGGESTION "Yes, pay now"
    ADD SUGGESTION "Schedule for due date"
    ADD SUGGESTION "Cancel"
    
    choice = HEAR
    
    IF choice CONTAINS "now" THEN
        ' Process payment
        result = ProcessBoletoPayment(boleto)
        
        IF result.success THEN
            TALK "✅ **Payment completed!**"
            TALK "Transaction ID: " + result.transaction_id
            TALK "Authentication: " + result.authentication
        ELSE
            TALK "❌ Payment failed: " + result.error
        END IF
        
    ELSEIF choice CONTAINS "schedule" THEN
        ' Schedule for due date
        TABLE scheduled
            ROW GenerateID(), GET USER ID, "boleto", barcode, boleto.amount, boleto.due_date, "pending"
        END TABLE
        
        SAVE "scheduled_payments.csv", scheduled
        
        TALK "✅ Payment scheduled for " + FORMAT(boleto.due_date, "dd/MM/yyyy")
    ELSE
        TALK "Payment cancelled."
    END IF
END SUB

' Pay utilities
SUB PayUtility(utility_type)
    TALK "Enter your " + utility_type + " account number or scan the bill:"
    account = HEAR
    
    ' Fetch bill info
    bill = FetchUtilityBill(utility_type, account)
    
    IF bill.found THEN
        TALK "📄 **" + utility_type + " Bill**"
        TALK "Account: " + account
        TALK "Reference: " + bill.reference
        TALK "Amount: R$ " + FORMAT(bill.amount, "0.00")
        TALK "Due date: " + FORMAT(bill.due_date, "dd/MM/yyyy")
        
        TALK "Pay this bill?"
        ' ... continue payment flow
    ELSE
        TALK "No pending bill found for this account."
    END IF
END SUB
```

### Loans

```basic
' dialogs/loan.bas

' Loan simulation
SUB SimulateLoan()
    TALK "Let's simulate a loan. What type of loan are you interested in?"
    
    ADD SUGGESTION "Personal Loan"
    ADD SUGGESTION "Payroll Loan"
    ADD SUGGESTION "Home Equity"
    ADD SUGGESTION "Vehicle Loan"
    
    loan_type = HEAR
    
    TALK "What amount do you need?"
    amount = HEAR
    amount = ParseMoney(amount)
    
    TALK "In how many months would you like to pay?"
    ADD SUGGESTION "12 months"
    ADD SUGGESTION "24 months"
    ADD SUGGESTION "36 months"
    ADD SUGGESTION "48 months"
    ADD SUGGESTION "60 months"
    
    months = HEAR
    months = ParseNumber(months)
    
    ' Get user's rate based on credit score
    user = GET USER
    rate = GetPersonalizedRate(user.id, loan_type)
    
    ' Calculate loan
    monthly_payment = CalculatePMT(amount, rate, months)
    total_amount = monthly_payment * months
    total_interest = total_amount - amount
    
    TALK "💰 **Loan Simulation**"
    TALK ""
    TALK "📊 **Summary**"
    TALK "Loan type: " + loan_type
    TALK "Amount: R$ " + FORMAT(amount, "0.00")
    TALK "Term: " + months + " months"
    TALK "Interest rate: " + FORMAT(rate * 100, "0.00") + "% per month"
    TALK ""
    TALK "📅 **Monthly Payment: R$ " + FORMAT(monthly_payment, "0.00") + "**"
    TALK ""
    TALK "Total to pay: R$ " + FORMAT(total_amount, "0.00")
    TALK "Total interest: R$ " + FORMAT(total_interest, "0.00")
    TALK ""
    TALK "Would you like to proceed with this loan?"
    
    ADD SUGGESTION "Yes, apply now"
    ADD SUGGESTION "Try different values"
    ADD SUGGESTION "Not now"
    
    choice = HEAR
    
    IF choice CONTAINS "apply" THEN
        CALL ApplyForLoan(loan_type, amount, months, rate)
    ELSEIF choice CONTAINS "different" THEN
        CALL SimulateLoan()
    ELSE
        TALK "No problem! I'm here whenever you need."
    END IF
END SUB

' Apply for loan
SUB ApplyForLoan(loan_type, amount, months, rate)
    user = GET USER
    
    ' Check eligibility
    eligibility = CheckLoanEligibility(user.id, loan_type, amount)
    
    IF NOT eligibility.eligible THEN
        TALK "❌ Unfortunately, we couldn't approve this loan at this time."
        TALK "Reason: " + eligibility.reason
        
        IF eligibility.alternative_amount > 0 THEN
            TALK "However, you're pre-approved for up to R$ " + FORMAT(eligibility.alternative_amount, "0.00")
            TALK "Would you like to apply for this amount instead?"
        END IF
        RETURN
    END IF
    
    TALK "✅ **Great news! You're pre-approved!**"
    TALK ""
    TALK "To complete your application, I need some additional information."
    
    ' Collect additional info
    TALK "What is your monthly income?"
    income = HEAR
    
    TALK "What is your profession?"
    profession = HEAR
    
    TALK "Do you have any other loans? (yes/no)"
    has_other_loans = HEAR
    
    IF has_other_loans CONTAINS "yes" THEN
        TALK "What is the total monthly payment of your other loans?"
        other_loans_payment = HEAR
    END IF
    
    ' Create loan application
    application_id = GenerateID()
    
    TABLE loan_application
        ROW application_id, user.id, loan_type, amount, months, rate, income, profession, NOW(), "pending_analysis"
    END TABLE
    
    SAVE "loan_applications.csv", loan_application
    
    TALK "🎉 **Application Submitted!**"
    TALK ""
    TALK "Application ID: " + application_id
    TALK "Status: Under Analysis"
    TALK ""
    TALK "We'll analyze your application within 24 hours."
    TALK "You'll receive updates via email and app notifications."
    
    ' Send notification
    SEND MAIL user.email, "Loan Application Received", "Your loan application " + application_id + " has been received and is under analysis."
END SUB
```

### Cards Management

```basic
' dialogs/cards.bas

' View cards
SUB ViewCards()
    user_id = GET USER ID
    
    cards = FIND "cards.csv" WHERE user_id = user_id AND status = "active"
    
    IF LEN(cards) = 0 THEN
        TALK "You don't have any active cards."
        TALK "Would you like to request one?"
        RETURN
    END IF
    
    TALK "💳 **Your Cards**"
    TALK ""
    
    FOR EACH card IN cards
        IF card.card_type = "credit" THEN
            icon = "💳"
        ELSE
            icon = "💵"
        END IF
        
        masked_number = "**** **** **** " + RIGHT(card.card_number, 4)
        
        TALK icon + " **" + card.card_type + " Card**"
        TALK "   Number: " + masked_number
        TALK "   Expiry: " + card.expiry_date
        
        IF card.card_type = "credit" THEN
            TALK "   Limit: R$ " + FORMAT(card.credit_limit, "0.00")
            TALK "   Available: R$ " + FORMAT(card.available_limit, "0.00")
            TALK "   Current bill: R$ " + FORMAT(card.current_bill, "0.00")
        END IF
        
        TALK "   Status: " + card.status
        TALK ""
    NEXT
    
    TALK "What would you like to do?"
    ADD SUGGESTION "View transactions"
    ADD SUGGESTION "Block card"
    ADD SUGGESTION "Request new card"
    ADD SUGGESTION "Increase limit"
END SUB

' Block card
SUB BlockCard(card_id)
    TALK "⚠️ **Block Card**"
    TALK "Are you sure you want to block this card?"
    TALK "This action will prevent all transactions."
    
    ADD SUGGESTION "Yes, block it"
    ADD SUGGESTION "Cancel"
    
    choice = HEAR
    
    IF choice CONTAINS "yes" THEN
        ' Request reason
        TALK "Please tell me why you're blocking the card:"
        ADD SUGGESTION "Lost"
        ADD SUGGESTION "Stolen"
        ADD SUGGESTION "Suspicious activity"
        ADD SUGGESTION "Temporary block"
        
        reason = HEAR
        
        ' Update card status
        UPDATE "cards.csv" SET status = "blocked", blocked_reason = reason WHERE id = card_id
        
        ' Log the action
        TABLE card_log
            ROW GenerateID(), card_id, "blocked", reason, NOW()
        END TABLE
        SAVE "card_logs.csv", card_log
        
        TALK "✅ **Card blocked successfully**"
        
        IF reason CONTAINS "stolen" OR reason CONTAINS "lost" THEN
            TALK "For your security, we recommend requesting a new card."
            TALK "Would you like to request a replacement?"
            
            IF HEAR CONTAINS "yes" THEN
                CALL RequestNewCard("replacement")
            END IF
        ELSE
            TALK "You can unblock your card anytime through this chat or the app."
        END IF
    ELSE
        TALK "Card block cancelled."
    END IF
END SUB

' Request credit limit increase
SUB RequestLimitIncrease()
    user_id = GET USER ID
    
    cards = FIND "cards.csv" WHERE user_id = user_id AND card_type = "credit" AND status = "active"
    
    IF LEN(cards) = 0 THEN
        TALK "You don't have an active credit card."
        RETURN
    END IF
    
    card = cards[0]
    current_limit = card.credit_limit
    
    ' Check eligibility for increase
    eligibility = CheckLimitIncreaseEligibility(card.id)
    
    IF eligibility.eligible THEN
        TALK "📈 **Good news! You're eligible for a limit increase!**"
        TALK ""
        TALK "Current limit: R$ " + FORMAT(current_limit, "0.00")
        TALK "Maximum available: R$ " + FORMAT(eligibility.max_limit, "0.00")
        TALK ""
        TALK "What limit would you like?"
        
        new_limit = HEAR
        new_limit = ParseMoney(new_limit)
        
        IF new_limit > eligibility.max_limit THEN
            TALK "The maximum limit available is R$ " + FORMAT(eligibility.max_limit, "0.00")
            new_limit = eligibility.max_limit
        END IF
        
        ' Approve instantly
        UPDATE "cards.csv" SET credit_limit = new_limit WHERE id = card.id
        
        TALK "✅ **Limit increased!**"
        TALK "New limit: R$ " + FORMAT(new_limit, "0.00")
        TALK "Effective immediately."
    ELSE
        TALK "At this time, we cannot increase your limit."
        TALK "Reason: " + eligibility.reason
        TALK "Please try again in " + eligibility.wait_days + " days."
    END IF
END SUB
```

### Investment Module

```basic
' dialogs/investment.bas

' View investments
SUB ViewInvestments()
    user_id = GET USER ID
    
    investments = FIND "investments.csv" WHERE user_id = user_id
    
    IF LEN(investments) = 0 THEN
        TALK "You don't have any investments yet."
        TALK "Would you like to explore our investment options?"
        
        IF HEAR CONTAINS "yes" THEN
            CALL ShowInvestmentOptions()
        END IF
        RETURN
    END IF
    
    total_invested = 0
    total_earnings = 0
    
    TALK "📊 **Your Investment Portfolio**"
    TALK ""
    
    FOR EACH inv IN investments
        earnings = inv.current_value - inv.invested_amount
        earnings_pct = (earnings / inv.invested_amount) * 100
        
        IF earnings >= 0 THEN
            icon = "📈"
            color = "green"
        ELSE
            icon = "📉"
            color = "red"
        END IF
        
        TALK icon + " **" + inv.product_name + "**"
        TALK "   Type: " + inv.product_type
        TALK "   Invested: R$ " + FORMAT(inv.invested_amount, "0.00")
        TALK "   Current: R$ " + FORMAT(inv.current_value, "0.00")
        TALK "   Return: " + FORMAT(earnings_pct, "0.00") + "%"
        TALK ""
        
        total_invested = total_invested + inv.invested_amount
        total_earnings = total_earnings + earnings
    NEXT
    
    total_pct = (total_earnings / total_invested) * 100
    
    TALK "💰 **Portfolio Summary**"
    TALK "Total invested: R$ " + FORMAT(total_invested, "0.00")
    TALK "Total value: R$ " + FORMAT(total_invested + total_earnings, "0.00")
    TALK "Total return: " + FORMAT(total_pct, "0.00") + "%"
END SUB

' Show investment options
SUB ShowInvestmentOptions()
    TALK "💎 **Investment Options**"
    TALK ""
    TALK "**Fixed Income:**"
    TALK "📌 CDB - from 100% CDI"
    TALK "📌 LCI/LCA - Tax-free, from 95% CDI"
    TALK "📌 Treasury Bonds - Government backed"
    TALK ""
    TALK "**Variable Income:**"
    TALK "📊 Stocks - Direct investment"
    TALK "📊 ETFs - Diversified funds"
    TALK "📊 REITs - Real estate funds"
    TALK ""
    TALK "**Crypto:**"
    TALK "🪙 Bitcoin, Ethereum, and more"
    TALK ""
    TALK "What interests you?"
    
    ADD SUGGESTION "Fixed Income"
    ADD SUGGESTION "Stocks"
    ADD SUGGESTION "Crypto"
    ADD SUGGESTION "I need advice"
END SUB
```

---

## 4. Excel Clone (HTMX/Rust)

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    GENERAL BOTS SHEETS                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    │
│  │   Browser   │◄──►│  HTMX/WS     │◄──►│  Rust Backend   │    │
│  │  (Canvas)   │    │  Updates     │    │  (Calamine)     │    │
│  └─────────────┘    └──────────────┘    └─────────────────┘    │
│         │                                       │               │
│         │                                       ▼               │
│         │                              ┌─────────────────┐     │
│         │                              │   File Storage   │     │
│         │                              │   (.gbdrive)     │     │
│         │                              └─────────────────┘     │
│         ▼                                                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    FORMULA ENGINE                        │   │
│  │  - 400+ Excel functions                                  │   │
│  │  - Array formulas                                        │   │
│  │  - Cross-sheet references                                │   │
│  │  - Custom functions (BASIC integration)                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Rust Backend

```rust
// src/sheets/mod.rs

use calamine::{Reader, Xlsx, DataType, Range};
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;

pub mod engine;
pub mod formulas;
pub mod api;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetState {
    pub id: Uuid,
    pub name: String,
    pub sheets: Vec<SheetState>,
    pub active_sheet: usize,
    pub modified: bool,
    pub last_saved: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetState {
    pub name: String,
    pub cells: HashMap<CellRef, CellData>,
    pub col_widths: HashMap<usize, f64>,
    pub row_heights: HashMap<usize, f64>,
    pub frozen_rows: usize,
    pub frozen_cols: usize,
    pub selection: Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRef {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    pub value: CellValue,
    pub formula: Option<String>,
    pub format: CellFormat,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValue {
    Empty,
    String(String),
    Number(f64),
    Boolean(bool),
    Error(String),
    DateTime(DateTime<Utc>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellFormat {
    pub number_format: String,
    pub alignment: Alignment,
    pub wrap_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellStyle {
    pub font: FontStyle,
    pub fill: FillStyle,
    pub border: BorderStyle,
}

// Spreadsheet Engine
pub struct SpreadsheetEngine {
    state: SpreadsheetState,
    formula_engine: FormulaEngine,
    dependency_graph: DependencyGraph,
}

impl SpreadsheetEngine {
    pub fn new() -> Self {
        Self {
            state: SpreadsheetState::default(),
            formula_engine: FormulaEngine::new(),
            dependency_graph: DependencyGraph::new(),
        }
    }

    pub fn load_xlsx(&mut self, path: &str) -> Result<(), Error> {
        let mut workbook: Xlsx<_> = calamine::open_workbook(path)?;
        
        for sheet_name in workbook.sheet_names().to_owned() {
            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                let mut sheet = SheetState::new(&sheet_name);
                
                for (row_idx, row) in range.rows().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        let cell_ref = CellRef { row: row_idx, col: col_idx };
                        let cell_data = self.convert_calamine_cell(cell);
                        sheet.cells.insert(cell_ref, cell_data);
                    }
                }
                
                self.state.sheets.push(sheet);
            }
        }
        
        Ok(())
    }

    pub fn save_xlsx(&self, path: &str) -> Result<(), Error> {
        let mut workbook = Workbook::new();
        
        for sheet in &self.state.sheets {
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(&sheet.name)?;
            
            for (cell_ref, cell_data) in &sheet.cells {
                match &cell_data.value {
                    CellValue::String(s) => {
                        worksheet.write_string(cell_ref.row as u32, cell_ref.col as u16, s)?;
                    }
                    CellValue::Number(n) => {
                        worksheet.write_number(cell_ref.row as u32, cell_ref.col as u16, *n)?;
                    }
                    CellValue::Boolean(b) => {
                        worksheet.write_boolean(cell_ref.row as u32, cell_ref.col as u16, *b)?;
                    }
                    _ => {}
                }
                
                // Write formula if exists
                if let Some(formula) = &cell_data.formula {
                    worksheet.write_formula(
                        cell_ref.row as u32, 
                        cell_ref.col as u16, 
                        formula
                    )?;
                }
            }
        }
        
        workbook.save(path)?;
        Ok(())
    }

    pub fn set_cell(&mut self, sheet: usize, row: usize, col: usize, value: &str) -> Vec<CellUpdate> {
        let cell_ref = CellRef { row, col };
        
        // Check if it's a formula
        if value.starts_with('=') {
            let formula = value[1..].to_string();
            let calculated = self.formula_engine.evaluate(&formula, &self.state.sheets[sheet]);
            
            self.state.sheets[sheet].cells.insert(cell_ref.clone(), CellData {
                value: calculated,
                formula: Some(formula),
                format: CellFormat::default(),
                style: CellStyle::default(),
            });
            
            // Update dependency graph
            self.dependency_graph.update(&cell_ref, &formula);
        } else {
            // Parse as value
            let cell_value = self.parse_value(value);
            
            self.state.sheets[sheet].cells.insert(cell_ref.clone(), CellData {
                value: cell_value,
                formula: None,
                format: CellFormat::default(),
                style: CellStyle::default(),
            });
        }
        
        // Recalculate dependents
        let updates = self.recalculate_dependents(&cell_ref);
        
        self.state.modified = true;
        updates
    }

    fn recalculate_dependents(&mut self, cell_ref: &CellRef) -> Vec<CellUpdate> {
        let mut updates = Vec::new();
        let dependents = self.dependency_graph.get_dependents(cell_ref);
        
        for dep in dependents {
            if let Some(cell) = self.state.sheets[self.state.active_sheet].cells.get_mut(&dep) {
                if let Some(formula) = &cell.formula {
                    let new_value = self.formula_engine.evaluate(
                        formula, 
                        &self.state.sheets[self.state.active_sheet]
                    );
                    cell.value = new_value.clone();
                    updates.push(CellUpdate {
                        row: dep.row,
                        col: dep.col,
                        value: new_value,
                    });
                }
            }
        }
        
        updates
    }
}
```

### Formula Engine

```rust
// src/sheets/formulas.rs

use std::collections::HashMap;

pub struct FormulaEngine {
    functions: HashMap<String, Box<dyn Fn(Vec<CellValue>) -> CellValue>>,
}

impl FormulaEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            functions: HashMap::new(),
        };
        engine.register_builtin_functions();
        engine
    }

    fn register_builtin_functions(&mut self) {
        // Math functions
        self.register("SUM", |args| {
            let sum: f64 = args.iter()
                .filter_map(|v| v.as_number())
                .sum();
            CellValue::Number(sum)
        });

        self.register("AVERAGE", |args| {
            let numbers: Vec<f64> = args.iter()
                .filter_map(|v| v.as_number())
                .collect();
            if numbers.is_empty() {
                CellValue::Error("#DIV/0!".to_string())
            } else {
                CellValue::Number(numbers.iter().sum::<f64>() / numbers.len() as f64)
            }
        });

        self.register("MIN", |args| {
            args.iter()
                .filter_map(|v| v.as_number())
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .map(CellValue::Number)
                .unwrap_or(CellValue::Error("#VALUE!".to_string()))
        });

        self.register("MAX", |args| {
            args.iter()
                .filter_map(|v| v.as_number())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .map(CellValue::Number)
                .unwrap_or(CellValue::Error("#VALUE!".to_string()))
        });

        self.register("COUNT", |args| {
            CellValue::Number(args.iter()
                .filter(|v| v.as_number().is_some())
                .count() as f64)
        });

        self.register("COUNTA", |args| {
            CellValue::Number(args.iter()
                .filter(|v| !matches!(v, CellValue::Empty))
                .count() as f64)
        });

        // Text functions
        self.register("CONCATENATE", |args| {
            let result: String = args.iter()
                .map(|v| v.to_string())
                .collect();
            CellValue::String(result)
        });

        self.register("LEFT", |args| {
            if args.len() >= 2 {
                let text = args[0].to_string();
                let n = args[1].as_number().unwrap_or(1.0) as usize;
                CellValue::String(text.chars().take(n).collect())
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("RIGHT", |args| {
            if args.len() >= 2 {
                let text = args[0].to_string();
                let n = args[1].as_number().unwrap_or(1.0) as usize;
                let start = text.len().saturating_sub(n);
                CellValue::String(text.chars().skip(start).collect())
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("MID", |args| {
            if args.len() >= 3 {
                let text = args[0].to_string();
                let start = (args[1].as_number().unwrap_or(1.0) as usize).saturating_sub(1);
                let n = args[2].as_number().unwrap_or(1.0) as usize;
                CellValue::String(text.chars().skip(start).take(n).collect())
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("LEN", |args| {
            if let Some(text) = args.get(0) {
                CellValue::Number(text.to_string().len() as f64)
            } else {
                CellValue::Number(0.0)
            }
        });

        self.register("TRIM", |args| {
            if let Some(text) = args.get(0) {
                CellValue::String(text.to_string().trim().to_string())
            } else {
                CellValue::String(String::new())
            }
        });

        self.register("UPPER", |args| {
            if let Some(text) = args.get(0) {
                CellValue::String(text.to_string().to_uppercase())
            } else {
                CellValue::String(String::new())
            }
        });

        self.register("LOWER", |args| {
            if let Some(text) = args.get(0) {
                CellValue::String(text.to_string().to_lowercase())
            } else {
                CellValue::String(String::new())
            }
        });

        // Logical functions
        self.register("IF", |args| {
            if args.len() >= 3 {
                let condition = args[0].as_bool().unwrap_or(false);
                if condition {
                    args[1].clone()
                } else {
                    args[2].clone()
                }
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("AND", |args| {
            CellValue::Boolean(args.iter().all(|v| v.as_bool().unwrap_or(false)))
        });

        self.register("OR", |args| {
            CellValue::Boolean(args.iter().any(|v| v.as_bool().unwrap_or(false)))
        });

        self.register("NOT", |args| {
            if let Some(val) = args.get(0) {
                CellValue::Boolean(!val.as_bool().unwrap_or(false))
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        // Lookup functions
        self.register("VLOOKUP", |args| {
            // Implementation for VLOOKUP
            CellValue::Error("#N/A".to_string()) // Placeholder
        });

        self.register("HLOOKUP", |args| {
            // Implementation for HLOOKUP
            CellValue::Error("#N/A".to_string()) // Placeholder
        });

        self.register("INDEX", |args| {
            // Implementation for INDEX
            CellValue::Error("#REF!".to_string()) // Placeholder
        });

        self.register("MATCH", |args| {
            // Implementation for MATCH
            CellValue::Error("#N/A".to_string()) // Placeholder
        });

        // Date functions
        self.register("TODAY", |_args| {
            CellValue::DateTime(Utc::now())
        });

        self.register("NOW", |_args| {
            CellValue::DateTime(Utc::now())
        });

        self.register("YEAR", |args| {
            if let Some(CellValue::DateTime(dt)) = args.get(0) {
                CellValue::Number(dt.year() as f64)
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("MONTH", |args| {
            if let Some(CellValue::DateTime(dt)) = args.get(0) {
                CellValue::Number(dt.month() as f64)
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        self.register("DAY", |args| {
            if let Some(CellValue::DateTime(dt)) = args.get(0) {
                CellValue::Number(dt.day() as f64)
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        // Financial functions
        self.register("PMT", |args| {
            if args.len() >= 3 {
                let rate = args[0].as_number().unwrap_or(0.0);
                let nper = args[1].as_number().unwrap_or(0.0);
                let pv = args[2].as_number().unwrap_or(0.0);
                
                if rate == 0.0 {
                    CellValue::Number(-pv / nper)
                } else {
                    let pmt = pv * rate * (1.0 + rate).powf(nper) / 
                              ((1.0 + rate).powf(nper) - 1.0);
                    CellValue::Number(-pmt)
                }
            } else {
                CellValue::Error("#VALUE!".to_string())
            }
        });

        // Add 400+ more functions...
    }

    fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Vec<CellValue>) -> CellValue + 'static,
    {
        self.functions.insert(name.to_uppercase(), Box::new(f));
    }

    pub fn evaluate(&self, formula: &str, sheet: &SheetState) -> CellValue {
        // Parse and evaluate formula
        let tokens = self.tokenize(formula);
        let ast = self.parse(tokens);
        self.eval_ast(&ast, sheet)
    }
}
```

### HTMX UI Component

```html
<!-- templates/sheets.html -->
{% extends "base.html" %}

{% block title %}Sheets - General Bots{% endblock %}

{% block content %}
<div class="sheets-container" id="sheets-app" hx-ext="ws" ws-connect="/ws/sheets">
    <!-- Toolbar -->
    <div class="sheets-toolbar">
        <div class="toolbar-section file-section">
            <button hx-post="/api/sheets/new" hx-target="#sheet-content">
                📄 New
            </button>
            <button onclick="openFile()">📂 Open</button>
            <button hx-post="/api/sheets/save" hx-swap="none">💾 Save</button>
            <button hx-get="/api/sheets/export?format=xlsx" hx-swap="none">
                ⬇️ Export
            </button>
        </div>
        
        <div class="toolbar-section edit-section">
            <button onclick="undo()">↩️</button>
            <button onclick="redo()">↪️</button>
            <button onclick="cut()">✂️</button>
            <button onclick="copy()">📋</button>
            <button onclick="paste()">📄</button>
        </div>
        
        <div class="toolbar-section format-section">
            <select id="font-family" onchange="setFontFamily(this.value)">
                <option value="Arial">Arial</option>
                <option value="Calibri">Calibri</option>
                <option value="Times New Roman">Times New Roman</option>
                <option value="Courier New">Courier New</option>
            </select>
            
            <select id="font-size" onchange="setFontSize(this.value)">
                <option value="8">8</option>
                <option value="10">10</option>
                <option value="11" selected>11</option>
                <option value="12">12</option>
                <option value="14">14</option>
                <option value="18">18</option>
                <option value="24">24</option>
            </select>
            
            <button onclick="toggleBold()"><b>B</b></button>
            <button onclick="toggleItalic()"><i>I</i></button>
            <button onclick="toggleUnderline()"><u>U</u></button>
            
            <input type="color" id="text-color" onchange="setTextColor(this.value)" value="#000000">
            <input type="color" id="fill-color" onchange="setFillColor(this.value)" value="#ffffff">
        </div>
        
        <div class="toolbar-section align-section">
            <button onclick="alignLeft()">⬅️</button>
            <button onclick="alignCenter()">↔️</button>
            <button onclick="alignRight()">➡️</button>
        </div>
        
        <div class="toolbar-section number-section">
            <select id="number-format" onchange="setNumberFormat(this.value)">
                <option value="general">General</option>
                <option value="number">Number</option>
                <option value="currency">Currency</option>
                <option value="percentage">Percentage</option>
                <option value="date">Date</option>
                <option value="time">Time</option>
                <option value="text">Text</option>
            </select>
        </div>
        
        <div class="toolbar-section ai-section">
            <button onclick="openAIAssist()" class="ai-button">
                🤖 AI Assist
            </button>
        </div>
    </div>
    
    <!-- Formula Bar -->
    <div class="formula-bar">
        <div class="cell-ref" id="cell-ref">A1</div>
        <div class="fx-label">fx</div>
        <input type="text" id="formula-input" class="formula-input"
               placeholder="Enter value or formula"
               onkeydown="handleFormulaInput(event)"
               hx-trigger="change"
               hx-post="/api/sheets/cell"
               hx-vals='js:{cell: getCellRef(), value: this.value}'
               hx-swap="none">
    </div>
    
    <!-- Spreadsheet Grid -->
    <div class="sheet-grid-container">
        <canvas id="sheet-canvas" 
                onmousedown="handleMouseDown(event)"
                onmousemove="handleMouseMove(event)"
                onmouseup="handleMouseUp(event)"
                ondblclick="handleDoubleClick(event)"
                oncontextmenu="handleContextMenu(event); return false;">
        </canvas>
        
        <!-- Cell Editor (shown on double-click) -->
        <input type="text" id="cell-editor" class="cell-editor hidden"
               onkeydown="handleCellEditorKey(event)"
               onblur="commitCellEdit()">
    </div>
    
    <!-- Sheet Tabs -->
    <div class="sheet-tabs">
        <div class="sheet-tab-list" id="sheet-tabs"
             hx-get="/api/sheets/tabs"
             hx-trigger="load"
             hx-swap="innerHTML">
            <!-- Tabs load here -->
        </div>
        <button class="add-sheet-btn" 
                hx-post="/api/sheets/add-sheet"
                hx-target="#sheet-tabs"
                hx-swap="beforeend">
            +
        </button>
    </div>
    
    <!-- Status Bar -->
    <div class="status-bar">
        <span id="selection-info">Ready</span>
        <span id="sum-info"></span>
        <span id="average-info"></span>
        <span id="count-info"></span>
    </div>
    
    <!-- Context Menu -->
    <div id="context-menu" class="context-menu hidden">
        <div onclick="cut()">✂️ Cut</div>
        <div onclick="copy()">📋 Copy</div>
        <div onclick="paste()">📄 Paste</div>
        <hr>
        <div onclick="insertRow()">Insert Row</div>
        <div onclick="insertColumn()">Insert Column</div>
        <div onclick="deleteRow()">Delete Row</div>
        <div onclick="deleteColumn()">Delete Column</div>
        <hr>
        <div onclick="formatCells()">Format Cells...</div>
    </div>
    
    <!-- AI Assistant Modal -->
    <div id="ai-modal" class="modal hidden">
        <div class="modal-content">
            <h3>🤖 AI Assistant</h3>
            <textarea id="ai-prompt" placeholder="Describe what you want to do...
Examples:
- Create a formula to sum column A
- Format as currency
- Create a pivot table from this data
- Generate sample data for testing"></textarea>
            <div class="modal-actions">
                <button onclick="closeAIModal()">Cancel</button>
                <button onclick="executeAICommand()" class="primary">Execute</button>
            </div>
        </div>
    </div>
</div>

<style>
.sheets-container {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 60px);
    background: white;
}

.sheets-toolbar {
    display: flex;
    gap: 16px;
    padding: 8px 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #f8f9fa;
    flex-wrap: wrap;
}

.toolbar-section {
    display: flex;
    gap: 4px;
    align-items: center;
    padding-right: 16px;
    border-right: 1px solid #e0e0e0;
}

.toolbar-section:last-child {
    border-right: none;
}

.toolbar-section button {
    padding: 6px 10px;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
}

.toolbar-section button:hover {
    background: #e8e8e8;
}

.formula-bar {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-bottom: 1px solid #e0e0e0;
    background: white;
}

.cell-ref {
    width: 80px;
    padding: 4px 8px;
    background: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: monospace;
    text-align: center;
}

.fx-label {
    padding: 0 8px;
    font-style: italic;
    color: #666;
}

.formula-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: 'Segoe UI', sans-serif;
}

.sheet-grid-container {
    flex: 1;
    overflow: hidden;
    position: relative;
}

#sheet-canvas {
    width: 100%;
    height: 100%;
}

.cell-editor {
    position: absolute;
    border: 2px solid #1a73e8;
    padding: 2px 4px;
    font-family: 'Segoe UI', sans-serif;
    font-size: 13px;
    outline: none;
    z-index: 100;
}

.sheet-tabs {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-top: 1px solid #e0e0e0;
    background: #f8f9fa;
}

.sheet-tab-list {
    display: flex;
    gap: 2px;
}

.sheet-tab {
    padding: 6px 16px;
    background: white;
    border: 1px solid #ddd;
    border-bottom: none;
    border-radius: 4px 4px 0 0;
    cursor: pointer;
}

.sheet-tab.active {
    background: #1a73e8;
    color: white;
}

.status-bar {
    display: flex;
    justify-content: space-between;
    padding: 4px 16px;
    background: #f0f0f0;
    border-top: 1px solid #ddd;
    font-size: 12px;
    color: #666;
}

.context-menu {
    position: fixed;
    background: white;
    border: 1px solid #ddd;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    z-index: 1000;
}

.context-menu div {
    padding: 8px 16px;
    cursor: pointer;
}

.context-menu div:hover {
    background: #f0f0f0;
}

.ai-button {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
    color: white !important;
    border: none !important;
}
</style>

<script>
// Spreadsheet rendering and interaction
const canvas = document.getElementById('sheet-canvas');
const ctx = canvas.getContext('2d');

const COL_WIDTH = 100;
const ROW_HEIGHT = 25;
const HEADER_HEIGHT = 25;
const ROW_HEADER_WIDTH = 50;

let cells = {};
let selection = { start: {row: 0, col: 0}, end: {row: 0, col: 0} };
let scrollOffset = { x: 0, y: 0 };
let isSelecting = false;

function resizeCanvas() {
    canvas.width = canvas.offsetWidth * window.devicePixelRatio;
    canvas.height = canvas.offsetHeight * window.devicePixelRatio;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    render();
}

function render() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;
    
    // Calculate visible range
    const startCol = Math.floor(scrollOffset.x / COL_WIDTH);
    const endCol = Math.ceil((scrollOffset.x + width - ROW_HEADER_WIDTH) / COL_WIDTH);
    const startRow = Math.floor(scrollOffset.y / ROW_HEIGHT);
    const endRow = Math.ceil((scrollOffset.y + height - HEADER_HEIGHT) / ROW_HEIGHT);
    
    // Draw column headers
    ctx.fillStyle = '#f8f9fa';
    ctx.fillRect(0, 0, width, HEADER_HEIGHT);
    ctx.fillStyle = '#333';
    ctx.font = '12px Segoe UI';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    
    for (let col = startCol; col <= endCol; col++) {
        const x = ROW_HEADER_WIDTH + col * COL_WIDTH - scrollOffset.x;
        ctx.fillText(colToLetter(col), x + COL_WIDTH / 2, HEADER_HEIGHT / 2);
        
        // Column border
        ctx.strokeStyle = '#e0e0e0';
        ctx.beginPath();
        ctx.moveTo(x + COL_WIDTH, 0);
        ctx.lineTo(x + COL_WIDTH, height);
        ctx.stroke();
    }
    
    // Draw row headers
    ctx.fillStyle = '#f8f9fa';
    ctx.fillRect(0, HEADER_HEIGHT, ROW_HEADER_WIDTH, height);
    ctx.fillStyle = '#333';
    ctx.textAlign = 'center';
    
    for (let row = startRow; row <= endRow; row++) {
        const y = HEADER_HEIGHT + row * ROW_HEIGHT - scrollOffset.y;
        ctx.fillText(String(row + 1), ROW_HEADER_WIDTH / 2, y + ROW_HEIGHT / 2);
        
        // Row border
        ctx.strokeStyle = '#e0e0e0';
        ctx.beginPath();
        ctx.moveTo(0, y + ROW_HEIGHT);
        ctx.lineTo(width, y + ROW_HEIGHT);
        ctx.stroke();
    }
    
    // Draw cells
    for (let row = startRow; row <= endRow; row++) {
        for (let col = startCol; col <= endCol; col++) {
            const x = ROW_HEADER_WIDTH + col * COL_WIDTH - scrollOffset.x;
            const y = HEADER_HEIGHT + row * ROW_HEIGHT - scrollOffset.y;
            
            const cellRef = `${colToLetter(col)}${row + 1}`;
            const cell = cells[cellRef];
            
            if (cell) {
                // Cell background
                if (cell.style?.fill) {
                    ctx.fillStyle = cell.style.fill;
                    ctx.fillRect(x + 1, y + 1, COL_WIDTH - 2, ROW_HEIGHT - 2);
                }
                
                // Cell text
                ctx.fillStyle = cell.style?.color || '#000';
                ctx.font = cell.style?.font || '13px Segoe UI';
                ctx.textAlign = cell.format?.alignment || 'left';
                ctx.textBaseline = 'middle';
                
                const textX = ctx.textAlign === 'left' ? x + 4 : 
                             ctx.textAlign === 'right' ? x + COL_WIDTH - 4 : 
                             x + COL_WIDTH / 2;
                
                ctx.fillText(formatCellValue(cell), textX, y + ROW_HEIGHT / 2);
            }
        }
    }
    
    // Draw selection
    drawSelection();
}

function drawSelection() {
    const startRow = Math.min(selection.start.row, selection.end.row);
    const endRow = Math.max(selection.start.row, selection.end.row);
    const startCol = Math.min(selection.start.col, selection.end.col);
    const endCol = Math.max(selection.start.col, selection.end.col);
    
    const x = ROW_HEADER_WIDTH + startCol * COL_WIDTH - scrollOffset.x;
    const y = HEADER_HEIGHT + startRow * ROW_HEIGHT - scrollOffset.y;
    const width = (endCol - startCol + 1) * COL_WIDTH;
    const height = (endRow - startRow + 1) * ROW_HEIGHT;
    
    // Selection fill
    ctx.fillStyle = 'rgba(26, 115, 232, 0.1)';
    ctx.fillRect(x, y, width, height);
    
    // Selection border
    ctx.strokeStyle = '#1a73e8';
    ctx.lineWidth = 2;
    ctx.strokeRect(x, y, width, height);
    ctx.lineWidth = 1;
}

function colToLetter(col) {
    let result = '';
    while (col >= 0) {
        result = String.fromCharCode(65 + (col % 26)) + result;
        col = Math.floor(col / 26) - 1;
    }
    return result;
}

function handleMouseDown(event) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    if (x > ROW_HEADER_WIDTH && y > HEADER_HEIGHT) {
        const col = Math.floor((x - ROW_HEADER_WIDTH + scrollOffset.x) / COL_WIDTH);
        const row = Math.floor((y - HEADER_HEIGHT + scrollOffset.y) / ROW_HEIGHT);
        
        selection.start = { row, col };
        selection.end = { row, col };
        isSelecting = true;
        
        updateCellRef();
        render();
    }
}

function handleMouseMove(event) {
    if (!isSelecting) return;
    
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    
    const col = Math.floor((x - ROW_HEADER_WIDTH + scrollOffset.x) / COL_WIDTH);
    const row = Math.floor((y - HEADER_HEIGHT + scrollOffset.y) / ROW_HEIGHT);
    
    selection.end = { row: Math.max(0, row), col: Math.max(0, col) };
    render();
}

function handleMouseUp() {
    isSelecting = false;
    updateSelectionInfo();
}

function handleDoubleClick(event) {
    const cellRef = getCellRef();
    showCellEditor(selection.start.row, selection.start.col);
}

function showCellEditor(row, col) {
    const editor = document.getElementById('cell-editor');
    const x = ROW_HEADER_WIDTH + col * COL_WIDTH - scrollOffset.x;
    const y = HEADER_HEIGHT + row * ROW_HEIGHT - scrollOffset.y;
    
    editor.style.left = x + 'px';
    editor.style.top = y + 'px';
    editor.style.width = COL_WIDTH + 'px';
    editor.style.height = ROW_HEIGHT + 'px';
    
    const cellRef = `${colToLetter(col)}${row + 1}`;
    const cell = cells[cellRef];
    editor.value = cell?.formula ? `=${cell.formula}` : (cell?.value || '');
    
    editor.classList.remove('hidden');
    editor.focus();
}

function commitCellEdit() {
    const editor = document.getElementById('cell-editor');
    const value = editor.value;
    const cellRef = getCellRef();
    
    // Send to server
    htmx.ajax('POST', '/api/sheets/cell', {
        values: { cell: cellRef, value: value }
    });
    
    editor.classList.add('hidden');
}

function getCellRef() {
    return `${colToLetter(selection.start.col)}${selection.start.row + 1}`;
}

function updateCellRef() {
    document.getElementById('cell-ref').textContent = getCellRef();
    
    const cellRef = getCellRef();
    const cell = cells[cellRef];
    const formulaInput = document.getElementById('formula-input');
    formulaInput.value = cell?.formula ? `=${cell.formula}` : (cell?.value || '');
}

// WebSocket for real-time updates
htmx.on('htmx:wsMessage', function(event) {
    const data = JSON.parse(event.detail.message);
    
    if (data.type === 'cell_update') {
        cells[data.cell] = data.data;
        render();
    }
});

// Initialize
window.addEventListener('resize', resizeCanvas);
resizeCanvas();
</script>
{% endblock %}
```

---

## 5. Word Editor for .docx

### Architecture

```rust
// src/docs/mod.rs

use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

pub struct DocumentEditor {
    document: Docx,
    file_path: Option<String>,
    modified: bool,
}

impl DocumentEditor {
    pub fn new() -> Self {
        Self {
            document: Docx::new(),
            file_path: None,
            modified: false,
        }
    }

    pub fn open(path: &str) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let document = docx_rs::read_docx(&file)?;
        
        Ok(Self {
            document,
            file_path: Some(path.to_string()),
            modified: false,
        })
    }

    pub fn save(&self, path: &str) -> Result<(), Error> {
        let file = std::fs::File::create(path)?;
        self.document.build().pack(file)?;
        Ok(())
    }

    pub fn add_paragraph(&mut self, text: &str, style: &ParagraphStyle) -> &mut Self {
        let mut paragraph = Paragraph::new();
        let mut run = Run::new().add_text(text);
        
        if style.bold {
            run = run.bold();
        }
        if style.italic {
            run = run.italic();
        }
        if let Some(size) = style.font_size {
            run = run.size(size * 2); // half-points
        }
        
        paragraph = paragraph.add_run(run);
        self.document = std::mem::take(&mut self.document).add_paragraph(paragraph);
        self.modified = true;
        self
    }

    pub fn to_html(&self) -> String {
        // Convert document to HTML for editing
        let mut html = String::new();
        // Implementation...
        html
    }

    pub fn from_html(&mut self, html: &str) -> Result<(), Error> {
        // Parse HTML and update document
        Ok(())
    }
}
```

### HTMX Word Editor UI

```html
<!-- templates/docs.html -->
{% extends "base.html" %}

{% block title %}Documents - General Bots{% endblock %}

{% block content %}
<div class="docs-container" id="docs-app" hx-ext="ws" ws-connect="/ws/docs">
    <!-- Toolbar -->
    <div class="docs-toolbar">
        <div class="toolbar-section">
            <button hx-post="/api/docs/new">📄 New</button>
            <button onclick="openDocument()">📂 Open</button>
            <button hx-post="/api/docs/save" hx-swap="none">💾 Save</button>
            <button hx-get="/api/docs/export?format=docx">⬇️ Export</button>
            <button hx-get="/api/docs/export?format=pdf">📑 PDF</button>
        </div>
        
        <div class="toolbar-section format-section">
            <select id="style-select" onchange="applyStyle(this.value)">
                <option value="normal">Normal</option>
                <option value="heading1">Heading 1</option>
                <option value="heading2">Heading 2</option>
                <option value="heading3">Heading 3</option>
                <option value="title">Title</option>
                <option value="subtitle">Subtitle</option>
            </select>
            
            <select id="font-family" onchange="setFont(this.value)">
                <option value="Calibri">Calibri</option>
                <option value="Arial">Arial</option>
                <option value="Times New Roman">Times New Roman</option>
                <option value="Georgia">Georgia</option>
            </select>
            
            <select id="font-size" onchange="setFontSize(this.value)">
                <option value="10">10</option>
                <option value="11">11</option>
                <option value="12" selected>12</option>
                <option value="14">14</option>
                <option value="16">16</option>
                <option value="18">18</option>
                <option value="24">24</option>
                <option value="36">36</option>
            </select>
        </div>
        
        <div class="toolbar-section">
            <button onclick="execCommand('bold')"><b>B</b></button>
            <button onclick="execCommand('italic')"><i>I</i></button>
            <button onclick="execCommand('underline')"><u>U</u></button>
            <button onclick="execCommand('strikeThrough')"><s>S</s></button>
        </div>
        
        <div class="toolbar-section">
            <button onclick="execCommand('justifyLeft')">⬅️</button>
            <button onclick="execCommand('justifyCenter')">↔️</button>
            <button onclick="execCommand('justifyRight')">➡️</button>
            <button onclick="execCommand('justifyFull')">☰</button>
        </div>
        
        <div class="toolbar-section">
            <button onclick="execCommand('insertUnorderedList')">• List</button>
            <button onclick="execCommand('insertOrderedList')">1. List</button>
            <button onclick="execCommand('indent')">→ Indent</button>
            <button onclick="execCommand('outdent')">← Outdent</button>
        </div>
        
        <div class="toolbar-section">
            <button onclick="insertTable()">📊 Table</button>
            <button onclick="insertImage()">🖼️ Image</button>
            <button onclick="insertLink()">🔗 Link</button>
        </div>
        
        <div class="toolbar-section ai-section">
            <button onclick="openAIWriter()" class="ai-button">
                🤖 AI Writer
            </button>
        </div>
    </div>
    
    <!-- Ruler -->
    <div class="ruler">
        <div class="ruler-marks"></div>
    </div>
    
    <!-- Document Canvas -->
    <div class="document-canvas">
        <div class="page" id="document-editor"
             contenteditable="true"
             hx-trigger="blur"
             hx-post="/api/docs/content"
             hx-swap="none"
             oninput="markModified()">
            <!-- Document content here -->
        </div>
    </div>
    
    <!-- Status Bar -->
    <div class="status-bar">
        <span id="page-info">Page 1 of 1</span>
        <span id="word-count">0 words</span>
        <span id="char-count">0 characters</span>
        <span id="save-status">Saved</span>
    </div>
    
    <!-- AI Writer Modal -->
    <div id="ai-writer-modal" class="modal hidden">
        <div class="modal-content large">
            <h3>🤖 AI Writer</h3>
            <div class="ai-options">
                <button onclick="aiAction('improve')">✨ Improve Writing</button>
                <button onclick="aiAction('shorten')">📝 Make Shorter</button>
                <button onclick="aiAction('expand')">📖 Expand</button>
                <button onclick="aiAction('formal')">👔 Make Formal</button>
                <button onclick="aiAction('casual')">😊 Make Casual</button>
                <button onclick="aiAction('translate')">🌐 Translate</button>
            </div>
            <textarea id="ai-prompt" placeholder="Or describe what you want..."></textarea>
            <div class="modal-actions">
                <button onclick="closeAIWriter()">Cancel</button>
                <button onclick="executeAI()" class="primary">Generate</button>
            </div>
        </div>
    </div>
</div>

<style>
.docs-container {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 60px);
    background: #525659;
}

.docs-toolbar {
    display: flex;
    gap: 12px;
    padding: 8px 16px;
    background: #f3f3f3;
    border-bottom: 1px solid #d6d6d6;
    flex-wrap: wrap;
}

.ruler {
    height: 24px;
    background: white;
    border-bottom: 1px solid #ddd;
}

.document-canvas {
    flex: 1;
    overflow: auto;
    padding: 40px;
    display: flex;
    justify-content: center;
}

.page {
    width: 8.5in;
    min-height: 11in;
    background: white;
    box-shadow: 0 2px 8px rgba(0,0,0,0.2);
    padding: 1in;
    font-family: 'Calibri', sans-serif;
    font-size: 12pt;
    line-height: 1.5;
    outline: none;
}

.page:focus {
    outline: none;
}

.status-bar {
    display: flex;
    justify-content: space-between;
    padding: 4px 16px;
    background: #f0f0f0;
    font-size: 12px;
    color: #666;
}

.ai-button {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
    color: white !important;
}

.ai-options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
}

.ai-options button {
    padding: 8px 16px;
    background: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 20px;
    cursor: pointer;
}

.ai-options button:hover {
    background: #e0e0e0;
}
</style>

<script>
function execCommand(command, value = null) {
    document.execCommand(command, false, value);
    document.getElementById('document-editor').focus();
}

function setFont(font) {
    execCommand('fontName', font);
}

function setFontSize(size) {
    execCommand('fontSize', size);
}

function applyStyle(style) {
    const selection = window.getSelection();
    if (!selection.rangeCount) return;
    
    let tag = 'p';
    switch (style) {
        case 'heading1': tag = 'h1'; break;
        case 'heading2': tag = 'h2'; break;
        case 'heading3': tag = 'h3'; break;
        case 'title': tag = 'h1'; break;
        case 'subtitle': tag = 'h2'; break;
    }
    
    execCommand('formatBlock', tag);
}

function insertTable() {
    const rows = prompt('Number of rows:', '3');
    const cols = prompt('Number of columns:', '3');
    
    if (rows && cols) {
        let html = '<table border="1" style="border-collapse: collapse; width: 100%;">';
        for (let r = 0; r < parseInt(rows); r++) {
            html += '<tr>';
            for (let c = 0; c < parseInt(cols); c++) {
                html += '<td style="padding: 8px; border: 1px solid #ddd;">&nbsp;</td>';
            }
            html += '</tr>';
        }
        html += '</table><p></p>';
        
        execCommand('insertHTML', html);
    }
}

function insertImage() {
    const url = prompt('Image URL:');
    if (url) {
        execCommand('insertImage', url);
    }
}

function insertLink() {
    const url = prompt('Link URL:');
    if (url) {
        execCommand('createLink', url);
    }
}

function markModified() {
    document.getElementById('save-status').textContent = 'Modified';
    updateWordCount();
}

function updateWordCount() {
    const text = document.getElementById('document-editor').innerText;
    const words = text.trim().split(/\s+/).filter(w => w.length > 0).length;
    const chars = text.length;
    
    document.getElementById('word-count').textContent = `${words} words`;
    document.getElementById('char-count').textContent = `${chars} characters`;
}

function openAIWriter() {
    document.getElementById('ai-writer-modal').classList.remove('hidden');
}

function closeAIWriter() {
    document.getElementById('ai-writer-modal').classList.add('hidden');
}

async function aiAction(action) {
    const selection = window.getSelection();
    const selectedText = selection.toString();
    
    if (!selectedText) {
        alert('Please select some text first');
        return;
    }
    
    const response = await fetch('/api/docs/ai', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, text: selectedText })
    });
    
    const result = await response.json();
    
    if (result.text) {
        execCommand('insertText', result.text);
    }
}
</script>
{% endblock %}
```

---

## 6. M365/Office Competitive Analysis

### Feature Comparison Matrix

| Feature | Microsoft 365 | Google Workspace | General Bots | Status |
|---------|---------------|------------------|--------------|--------|
| **Email** | Outlook | Gmail | ✅ Mail | Complete |
| **Calendar** | Outlook Calendar | Google Calendar | ✅ Calendar | Complete |
| **File Storage** | OneDrive | Google Drive | ✅ .gbdrive | Complete |
| **Word Processing** | Word | Docs | 🔄 Docs Editor | In Progress |
| **Spreadsheets** | Excel | Sheets | 🔄 Sheets Editor | In Progress |
| **Presentations** | PowerPoint | Slides | 📋 Planned | Planned |
| **Video Calls** | Teams | Meet | 🔄 Meet | In Progress |
| **Chat** | Teams Chat | Google Chat | ✅ Chat | Complete |
| **AI Assistant** | Copilot | Gemini | ✅ Multi-LLM | Complete |
| **Tasks** | To Do/Planner | Tasks | ✅ Tasks | Complete |
| **Forms** | Forms | Forms | ✅ Forms | Complete |
| **Notes** | OneNote | Keep | 📋 Planned | Planned |
| **Whiteboard** | Whiteboard | Jamboard | 📋 Planned | Planned |

### Missing Features to Implement

```rust
// Priority 1: Core Office Features
// - Presentations engine (PowerPoint/Slides equivalent)
// - Real-time collaboration (multiple users editing)
// - Version history and restore
// - Comments and suggestions mode

// Priority 2: Copilot/Gemini Parity
// - AI in documents (rewrite, summarize, expand)
// - AI in spreadsheets (formula generation, data analysis)
// - AI in email (compose, reply, summarize threads)
// - AI in meetings (transcription, summary, action items)

// Priority 3: Enterprise Features
// - Admin console
// - Compliance center (eDiscovery, legal hold)
// - Data loss prevention
// - Retention policies
// - Audit logs (already have basic)
```

---

## 7. Google/MS Graph API Compatibility

### API Endpoints to Implement

```rust
// src/api/compat/google.rs

// Google Drive API compatible endpoints
// GET  /drive/v3/files
// POST /drive/v3/files
// GET  /drive/v3/files/{fileId}
// DELETE /drive/v3/files/{fileId}
// PATCH /drive/v3/files/{fileId}

// Google Calendar API compatible endpoints
// GET  /calendar/v3/calendars/{calendarId}/events
// POST /calendar/v3/calendars/{calendarId}/events
// GET  /calendar/v3/calendars/{calendarId}/events/{eventId}

// Google Gmail API compatible endpoints
// GET  /gmail/v1/users/{userId}/messages
// POST /gmail/v1/users/{userId}/messages/send
// GET  /gmail/v1/users/{userId}/threads

// src/api/compat/msgraph.rs

// Microsoft Graph API compatible endpoints
// GET  /v1.0/me/drive/root/children
// GET  /v1.0/me/messages
// POST /v1.0/me/sendMail
// GET  /v1.0/me/calendar/events
// POST /v1.0/me/calendar/events
// GET  /v1.0/me/contacts

pub fn configure_compat_routes(cfg: &mut web::ServiceConfig) {
    // Google API compatibility
    cfg.service(
        web::scope("/drive/v3")
            .route("/files", web::get().to(google_list_files))
            .route("/files", web::post().to(google_create_file))
            .route("/files/{fileId}", web::get().to(google_get_file))
    );
    
    // MS Graph API compatibility
    cfg.service(
        web::scope("/v1.0")
            .route("/me/drive/root/children", web::get().to(graph_list_files))
            .route("/me/messages", web::get().to(graph_list_messages))
            .route("/me/sendMail", web::post().to(graph_send_mail))
    );
}
```

---

## 8. Copilot/Gemini Feature Parity

### AI Features Checklist

| Feature | Copilot | Gemini | General Bots | BASIC Keyword |
|---------|---------|--------|--------------|---------------|
| Chat with AI | ✅ | ✅ | ✅ | `LLM` |
| Web search | ✅ | ✅ | 📋 | `SEARCH WEB` |
| Image generation | ✅ | ✅ | ✅ | `IMAGE` |
| Code generation | ✅ | ✅ | ✅ | `LLM` |
| Document summary | ✅ | ✅ | ✅ | `LLM` with file |
| Email compose | ✅ | ✅ | ✅ | `SEND MAIL` |
| Meeting summary | ✅ | ✅ | 📋 | `SUMMARIZE MEETING` |
| Data analysis | ✅ | ✅ | ✅ | `AGGREGATE` |
| Create presentations | ✅ | ✅ | 📋 | `CREATE PPT` |
| Voice input | ✅ | ✅ | ✅ | Voice API |
| Multi-modal | ✅ | ✅ | ✅ | `SEE`, `IMAGE` |
| Tool use | ✅ | ✅ | ✅ | `USE TOOL` |
| Memory/context | ✅ | ✅ | ✅ | `SET CONTEXT` |
| Multi-turn | ✅ | ✅ | ✅ | Built-in |

---

## 9. Attachment System (Plus Button)

### Implementation

```rust
// src/api/attachments.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub file_type: AttachmentType,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    Document,
    Audio,
    Video,
    Code,
    Archive,
    Other,
}

pub async fn upload_attachment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserSession>,
    mut multipart: Multipart,
) -> Result<Json<Attachment>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await?;
        
        // Determine attachment type
        let file_type = detect_attachment_type(&content_type, &file_name);
        
        // Store file
        let storage_path = store_attachment(&state, &user, &data, &file_name).await?;
        
        // Generate thumbnail for images/videos
        let thumbnail_path = if matches!(file_type, AttachmentType::Image | AttachmentType::Video) {
            Some(generate_thumbnail(&storage_path).await?)
        } else {
            None
        };
        
        // Create attachment record
        let attachment = Attachment {
            id: Uuid::new_v4(),
            message_id: None,
            file_type,
            file_name,
            file_size: data.len() as i64,
            mime_type: content_type,
            storage_path,
            thumbnail_path,
            created_at: Utc::now(),
        };
        
        // Save to database
        save_attachment(&state, &attachment).await?;
        
        return Ok(Json(attachment));
    }
    
    Err(ApiError::BadRequest("No file provided".to_string()))
}
```

---

## 10. Conversation Branching

### Database Schema

```sql
-- Conversation branches
CREATE TABLE conversation_branches (
    id UUID PRIMARY KEY,
    parent_session_id UUID NOT NULL,
    branch_session_id UUID NOT NULL,
    branch_from_message_id UUID NOT NULL,
    branch_name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (parent_session_id) REFERENCES sessions(id),
    FOREIGN KEY (branch_session_id) REFERENCES sessions(id),
    FOREIGN KEY (branch_from_message_id) REFERENCES messages(id)
);
```

### Implementation

```rust
// src/api/branches.rs

pub async fn create_branch(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<UserSession>,
    Json(req): Json<CreateBranchRequest>,
) -> Result<Json<BranchResponse>, ApiError> {
    // Create new session for branch
    let branch_session = create_session(&state, user.user_id, user.bot_id).await?;
    
    // Copy messages up to branch point
    copy_messages_to_branch(
        &state,
        user.id,
        branch_session.id,
        req.branch_from_message_id,
    ).await?;
    
    // Create branch record
    let branch = ConversationBranch {
        id: Uuid::new_v4(),
        parent_session_id: user.id,
        branch_session_id: branch_session.id,
        branch_from_message_id: req.branch_from_message_id,
        branch_name: req.name,
        created_at: Utc::now(),
    };
    
    save_branch(&state, &branch).await?;
    
    Ok(Json(BranchResponse {
        branch_id: branch.id,
        session_id: branch_session.id,
    }))
}
```

### UI Component

```html
<!-- Message with branch option -->
<div class="message" data-message-id="{{ message.id }}">
    <div class="message-content">{{ message.content }}</div>
    <div class="message-actions">
        <button onclick="branchFromMessage('{{ message.id }}')" title="Create branch">
            🌿
        </button>
        <button onclick="copyMessage('{{ message.id }}')" title="Copy">
            📋
        </button>
    </div>
</div>

<script>
async function branchFromMessage(messageId) {
    const name = prompt('Name for this branch:', 'Branch ' + new Date().toLocaleString());
    if (!name) return;
    
    const response = await fetch('/api/chat/branch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            branch_from