# Chapter 04: User Interface Architecture - Web Components and Real-Time Communication

The General Bots User Interface (gbui) system implements a sophisticated, component-based architecture for creating responsive, real-time conversational experiences across multiple platforms and devices. This chapter provides comprehensive technical documentation on the UI framework, rendering pipeline, WebSocket communication protocols, and interface customization capabilities.

## Executive Summary

The gbui system represents a modern approach to conversational UI development, implementing a lightweight, standards-based architecture that eliminates framework dependencies while providing enterprise-grade capabilities. The system leverages native Web Components, WebSocket protocols, and progressive enhancement strategies to deliver sub-second response times and seamless user experiences across desktop, mobile, and embedded platforms.

## System Architecture Overview

### UI Component Stack

The user interface implements a layered architecture for maximum flexibility and performance:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Presentation Layer                           │
│              (HTML5, CSS3, Web Components)                      │
├─────────────────────────────────────────────────────────────────┤
│                    Interaction Layer                            │
│         (Event Handling, Gesture Recognition, A11y)             │
├─────────────────────────────────────────────────────────────────┤
│                  Communication Layer                            │
│      (WebSocket, Server-Sent Events, REST API)                 │
├─────────────────────────────────────────────────────────────────┤
│                     State Management                            │
│        (Session Storage, IndexedDB, Service Workers)            │
├─────────────────────────────────────────────────────────────────┤
│                   Rendering Pipeline                            │
│      (Virtual DOM, Incremental Updates, GPU Acceleration)       │
└─────────────────────────────────────────────────────────────────┘
```

### Technical Specifications

| Component | Technology | Performance Target | Browser Support |
|-----------|------------|-------------------|-----------------|
| Rendering Engine | Native DOM + Virtual DOM diffing | 60 FPS animations | Chrome 90+, Firefox 88+, Safari 14+ |
| Communication | WebSocket (RFC 6455) | <100ms latency | All modern browsers |
| State Management | IndexedDB + LocalStorage | <5ms read/write | All modern browsers |
| Component System | Web Components v1 | <50ms initialization | Chrome, Firefox, Safari, Edge |
| Styling Engine | CSS Grid + Flexbox + Custom Properties | <16ms paint | All modern browsers |
| Build Size | Vanilla JS (no framework) | <50KB gzipped | IE11+ with polyfills |

## Template Architecture

### Template Processing Pipeline

The gbui template system implements a multi-stage processing pipeline:

```javascript
class TemplateProcessor {
    /**
     * Advanced template processing with optimization
     */
    constructor() {
        this.templateCache = new Map();
        this.componentRegistry = new Map();
        this.renderQueue = [];
        this.rafId = null;
    }
    
    processTemplate(templatePath, data) {
        // Stage 1: Template loading and caching
        const template = this.loadTemplate(templatePath);
        
        // Stage 2: Template parsing and AST generation
        const ast = this.parseTemplate(template);
        
        // Stage 3: Data binding and interpolation
        const boundAST = this.bindData(ast, data);
        
        // Stage 4: Component resolution
        const resolvedAST = this.resolveComponents(boundAST);
        
        // Stage 5: Optimization pass
        const optimizedAST = this.optimizeAST(resolvedAST);
        
        // Stage 6: DOM generation
        const domFragment = this.generateDOM(optimizedAST);
        
        // Stage 7: Hydration and event binding
        this.hydrate(domFragment);
        
        return domFragment;
    }
    
    parseTemplate(template) {
        /**
         * Convert HTML template to Abstract Syntax Tree
         */
        const parser = new DOMParser();
        const doc = parser.parseFromString(template, 'text/html');
        
        return this.buildAST(doc.body);
    }
    
    buildAST(node) {
        const ast = {
            type: node.nodeType === 1 ? 'element' : 'text',
            tag: node.tagName?.toLowerCase(),
            attributes: {},
            children: [],
            directives: {},
            events: {}
        };
        
        if (node.nodeType === 1) {
            // Process attributes
            for (const attr of node.attributes) {
                if (attr.name.startsWith('data-')) {
                    ast.directives[attr.name.slice(5)] = attr.value;
                } else if (attr.name.startsWith('on')) {
                    ast.events[attr.name.slice(2)] = attr.value;
                } else {
                    ast.attributes[attr.name] = attr.value;
                }
            }
            
            // Process children
            for (const child of node.childNodes) {
                if (child.nodeType === 1 || child.nodeType === 3) {
                    ast.children.push(this.buildAST(child));
                }
            }
        } else if (node.nodeType === 3) {
            ast.content = node.textContent;
        }
        
        return ast;
    }
}
```

### Template Types and Specifications

#### default.gbui - Full Desktop Interface

Complete workspace implementation with modular applications:

```html
<!DOCTYPE html>
<html lang="en" data-theme="system">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="General Bots Workspace Interface">
    
    <!-- Performance optimizations -->
    <link rel="preconnect" href="wss://localhost:8080">
    <link rel="dns-prefetch" href="//localhost:8080">
    
    <!-- Critical CSS inline for faster FCP -->
    <style>
        /* Critical path CSS */
        :root {
            --color-primary: #007bff;
            --color-background: #ffffff;
            --color-surface: #f8f9fa;
            --transition-speed: 200ms;
        }
        
        body {
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto;
            background: var(--color-background);
        }
        
        .app-container {
            display: grid;
            grid-template-areas:
                "header header header"
                "sidebar main aside"
                "footer footer footer";
            grid-template-columns: 240px 1fr 320px;
            grid-template-rows: 48px 1fr 32px;
            height: 100vh;
        }
        
        @media (max-width: 768px) {
            .app-container {
                grid-template-areas:
                    "header"
                    "main"
                    "footer";
                grid-template-columns: 1fr;
            }
        }
    </style>
    
    <!-- Async load non-critical CSS -->
    <link rel="stylesheet" href="/css/components.css" media="print" onload="this.media='all'">
    <link rel="stylesheet" href="/css/themes.css" media="print" onload="this.media='all'">
</head>
<body>
    <!-- Application Shell -->
    <div class="app-container" id="app">
        <!-- Header Bar -->
        <header class="app-header" role="banner">
            <div class="header-brand">
                <img src="/assets/logo.svg" alt="General Bots" width="32" height="32">
                <h1 class="header-title">General Bots</h1>
            </div>
            
            <nav class="header-nav" role="navigation" aria-label="Main">
                <button class="nav-item" data-app="chat" aria-label="Chat" aria-keyshortcuts="Alt+1">
                    <svg class="icon" aria-hidden="true"><!-- Chat icon --></svg>
                    <span>Chat</span>
                </button>
                <button class="nav-item" data-app="drive" aria-label="Drive" aria-keyshortcuts="Alt+2">
                    <svg class="icon" aria-hidden="true"><!-- Drive icon --></svg>
                    <span>Drive</span>
                </button>
                <button class="nav-item" data-app="tasks" aria-label="Tasks" aria-keyshortcuts="Alt+3">
                    <svg class="icon" aria-hidden="true"><!-- Tasks icon --></svg>
                    <span>Tasks</span>
                </button>
                <button class="nav-item" data-app="mail" aria-label="Mail" aria-keyshortcuts="Alt+4">
                    <svg class="icon" aria-hidden="true"><!-- Mail icon --></svg>
                    <span>Mail</span>
                </button>
            </nav>
            
            <div class="header-actions">
                <button class="theme-toggle" aria-label="Toggle theme">
                    <svg class="icon" aria-hidden="true"><!-- Theme icon --></svg>
                </button>
                <button class="user-menu" aria-label="User menu">
                    <img src="/api/user/avatar" alt="" class="avatar" width="32" height="32">
                </button>
            </div>
        </header>
        
        <!-- Sidebar -->
        <aside class="app-sidebar" role="complementary">
            <nav class="sidebar-nav" aria-label="Secondary">
                <!-- Dynamic navigation items -->
            </nav>
        </aside>
        
        <!-- Main Content Area -->
        <main class="app-main" role="main">
            <div class="app-view" id="app-view">
                <!-- Dynamic content loaded here -->
            </div>
        </main>
        
        <!-- Right Panel -->
        <aside class="app-aside" role="complementary">
            <div class="panel-content">
                <!-- Contextual information -->
            </div>
        </aside>
        
        <!-- Footer -->
        <footer class="app-footer" role="contentinfo">
            <div class="footer-status">
                <span class="status-indicator" aria-live="polite"></span>
                <span class="connection-status">Connected</span>
            </div>
        </footer>
    </div>
    
    <!-- Web Components -->
    <script type="module">
        // Chat Component
        class ChatComponent extends HTMLElement {
            constructor() {
                super();
                this.attachShadow({ mode: 'open' });
                this.messages = [];
                this.ws = null;
            }
            
            connectedCallback() {
                this.render();
                this.connectWebSocket();
                this.setupEventListeners();
            }
            
            render() {
                this.shadowRoot.innerHTML = `
                    <style>
                        :host {
                            display: flex;
                            flex-direction: column;
                            height: 100%;
                        }
                        
                        .messages {
                            flex: 1;
                            overflow-y: auto;
                            padding: 1rem;
                        }
                        
                        .message {
                            margin: 0.5rem 0;
                            padding: 0.75rem;
                            border-radius: 8px;
                            max-width: 70%;
                            animation: fadeIn 0.3s ease;
                        }
                        
                        .message-user {
                            background: var(--color-primary);
                            color: white;
                            align-self: flex-end;
                            margin-left: auto;
                        }
                        
                        .message-bot {
                            background: var(--color-surface);
                            align-self: flex-start;
                        }
                        
                        .input-area {
                            display: flex;
                            padding: 1rem;
                            border-top: 1px solid var(--color-border);
                        }
                        
                        @keyframes fadeIn {
                            from { opacity: 0; transform: translateY(10px); }
                            to { opacity: 1; transform: translateY(0); }
                        }
                    </style>
                    
                    <div class="messages" role="log" aria-live="polite"></div>
                    
                    <form class="input-area">
                        <input 
                            type="text" 
                            class="message-input"
                            placeholder="Type a message..."
                            aria-label="Message input"
                            autocomplete="off"
                        />
                        <button type="submit" aria-label="Send message">
                            <svg class="icon"><!-- Send icon --></svg>
                        </button>
                    </form>
                `;
            }
            
            connectWebSocket() {
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                this.ws = new WebSocket(`${protocol}//${window.location.host}/ws`);
                
                this.ws.onopen = () => {
                    console.log('WebSocket connected');
                    this.dispatchEvent(new CustomEvent('connected'));
                };
                
                this.ws.onmessage = (event) => {
                    const message = JSON.parse(event.data);
                    this.addMessage(message);
                };
                
                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    this.reconnect();
                };
            }
            
            addMessage(message) {
                this.messages.push(message);
                this.renderMessage(message);
                this.scrollToBottom();
            }
        }
        
        // Register Web Component
        customElements.define('chat-component', ChatComponent);
    </script>
</body>
</html>
```

#### single.gbui - Minimalist Chat Interface

Lightweight, focused chat implementation:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>Chat</title>
    
    <style>
        /* Minimal critical CSS - 3KB gzipped */
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: system-ui, -apple-system, sans-serif;
            height: 100vh;
            display: flex;
            flex-direction: column;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        
        .chat-container {
            flex: 1;
            display: flex;
            flex-direction: column;
            max-width: 800px;
            width: 100%;
            margin: 0 auto;
            background: white;
            box-shadow: 0 0 40px rgba(0,0,0,0.1);
        }
        
        .messages {
            flex: 1;
            overflow-y: auto;
            padding: 2rem;
            scroll-behavior: smooth;
        }
        
        .message {
            margin: 1rem 0;
            display: flex;
            animation: slideUp 0.3s ease;
        }
        
        .message-content {
            padding: 0.75rem 1rem;
            border-radius: 18px;
            max-width: 75%;
            word-wrap: break-word;
        }
        
        .user .message-content {
            background: #007bff;
            color: white;
            margin-left: auto;
            border-bottom-right-radius: 4px;
        }
        
        .bot .message-content {
            background: #f1f3f5;
            color: #333;
            border-bottom-left-radius: 4px;
        }
        
        .typing-indicator {
            display: none;
            padding: 0.75rem 1rem;
            background: #f1f3f5;
            border-radius: 18px;
            width: 60px;
        }
        
        .typing-indicator.active {
            display: inline-block;
        }
        
        .typing-indicator span {
            display: inline-block;
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: #999;
            margin: 0 2px;
            animation: typing 1.4s infinite;
        }
        
        .typing-indicator span:nth-child(2) {
            animation-delay: 0.2s;
        }
        
        .typing-indicator span:nth-child(3) {
            animation-delay: 0.4s;
        }
        
        .input-area {
            display: flex;
            padding: 1rem;
            background: white;
            border-top: 1px solid #e9ecef;
        }
        
        .message-input {
            flex: 1;
            padding: 0.75rem;
            border: 1px solid #dee2e6;
            border-radius: 24px;
            font-size: 1rem;
            outline: none;
            transition: border-color 0.2s;
        }
        
        .message-input:focus {
            border-color: #007bff;
        }
        
        .send-button {
            margin-left: 0.5rem;
            padding: 0.75rem;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 50%;
            width: 48px;
            height: 48px;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            transition: background 0.2s;
        }
        
        .send-button:hover {
            background: #0056b3;
        }
        
        .send-button:disabled {
            background: #6c757d;
            cursor: not-allowed;
        }
        
        @keyframes slideUp {
            from {
                opacity: 0;
                transform: translateY(20px);
            }
            to {
                opacity: 1;
                transform: translateY(0);
            }
        }
        
        @keyframes typing {
            0%, 60%, 100% {
                transform: translateY(0);
            }
            30% {
                transform: translateY(-10px);
            }
        }
        
        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
            body {
                background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            }
            
            .chat-container {
                background: #1e1e1e;
            }
            
            .bot .message-content {
                background: #2d2d2d;
                color: #e0e0e0;
            }
            
            .input-area {
                background: #1e1e1e;
                border-top-color: #333;
            }
            
            .message-input {
                background: #2d2d2d;
                border-color: #404040;
                color: #e0e0e0;
            }
        }
        
        /* Mobile optimizations */
        @media (max-width: 768px) {
            .chat-container {
                border-radius: 0;
            }
            
            .messages {
                padding: 1rem;
            }
            
            .message-content {
                max-width: 85%;
            }
        }
    </style>
</head>
<body>
    <div class="chat-container">
        <div class="messages" id="messages">
            <!-- Messages will be added here -->
        </div>
        
        <div class="message bot">
            <div class="typing-indicator" id="typing">
                <span></span>
                <span></span>
                <span></span>
            </div>
        </div>
        
        <form class="input-area" id="chat-form">
            <input 
                type="text" 
                class="message-input" 
                id="message-input"
                placeholder="Type your message..."
                autocomplete="off"
                autofocus
            />
            <button type="submit" class="send-button" id="send-button">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="22" y1="2" x2="11" y2="13"></line>
                    <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
                </svg>
            </button>
        </form>
    </div>
    
    <script>
        // Lightweight chat implementation - 5KB uncompressed
        (function() {
            'use strict';
            
            const elements = {
                messages: document.getElementById('messages'),
                input: document.getElementById('message-input'),
                form: document.getElementById('chat-form'),
                typing: document.getElementById('typing'),
                sendButton: document.getElementById('send-button')
            };
            
            let ws = null;
            let reconnectTimeout = null;
            let isConnected = false;
            
            // Initialize WebSocket connection
            function connect() {
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                ws = new WebSocket(`${protocol}//${window.location.host}/ws/chat`);
                
                ws.onopen = () => {
                    isConnected = true;
                    clearTimeout(reconnectTimeout);
                    console.log('Connected to chat server');
                    
                    // Send initial handshake
                    ws.send(JSON.stringify({
                        type: 'handshake',
                        timestamp: Date.now()
                    }));
                };
                
                ws.onmessage = (event) => {
                    const data = JSON.parse(event.data);
                    handleMessage(data);
                };
                
                ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                };
                
                ws.onclose = () => {
                    isConnected = false;
                    console.log('Disconnected from chat server');
                    
                    // Attempt reconnection
                    reconnectTimeout = setTimeout(connect, 3000);
                };
            }
            
            // Handle incoming messages
            function handleMessage(data) {
                switch(data.type) {
                    case 'message':
                        addMessage(data.content, data.sender || 'bot');
                        break;
                        
                    case 'typing':
                        showTypingIndicator(data.isTyping);
                        break;
                        
                    case 'error':
                        console.error('Server error:', data.message);
                        break;
                }
            }
            
            // Add message to chat
            function addMessage(content, sender) {
                const messageDiv = document.createElement('div');
                messageDiv.className = `message ${sender}`;
                
                const contentDiv = document.createElement('div');
                contentDiv.className = 'message-content';
                contentDiv.textContent = content;
                
                messageDiv.appendChild(contentDiv);
                elements.messages.appendChild(messageDiv);
                
                // Scroll to bottom
                elements.messages.scrollTop = elements.messages.scrollHeight;
                
                // Hide typing indicator
                showTypingIndicator(false);
            }
            
            // Show/hide typing indicator
            function showTypingIndicator(show) {
                elements.typing.classList.toggle('active', show);
                if (show) {
                    elements.messages.scrollTop = elements.messages.scrollHeight;
                }
            }
            
            // Send message
            function sendMessage(content) {
                if (!content.trim() || !isConnected) return;
                
                // Add user message to UI
                addMessage(content, 'user');
                
                // Send to server
                ws.send(JSON.stringify({
                    type: 'message',
                    content: content,
                    timestamp: Date.now()
                }));
                
                // Show typing indicator
                showTypingIndicator(true);
                
                // Clear input
                elements.input.value = '';
                elements.input.focus();
            }
            
            // Handle form submission
            elements.form.addEventListener('submit', (e) => {
                e.preventDefault();
                sendMessage(elements.input.value);
            });
            
            // Handle Enter key
            elements.input.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    sendMessage(elements.input.value);
                }
            });
            
            // Initialize connection
            connect();
            
            // Add welcome message
            setTimeout(() => {
                addMessage('Hello! How can I help you today?', 'bot');
            }, 500);
        })();
    </script>
</body>
</html>
```

## WebSocket Communication Protocol

### Protocol Specification

Real-time bidirectional communication implementation:

```typescript
interface WebSocketProtocol {
    // Message Types
    enum MessageType {
        HANDSHAKE = 'handshake',
        MESSAGE = 'message',
        TYPING = 'typing',
        PRESENCE = 'presence',
        ERROR = 'error',
        HEARTBEAT = 'heartbeat',
        ACKNOWLEDGE = 'acknowledge'
    }
    
    // Base Message Structure
    interface BaseMessage {
        id: string;
        type: MessageType;
        timestamp: number;
        version: string;
    }
    
    // Message Implementations
    interface ChatMessage extends BaseMessage {
        type: MessageType.MESSAGE;
        content: string;
        sender: 'user' | 'bot';
        metadata?: {
            tokens?: number;
            processingTime?: number;
            confidence?: number;
            sources?: string[];
        };
    }
    
    interface TypingMessage extends BaseMessage {
        type: MessageType.TYPING;
        isTyping: boolean;
        sender: string;
    }
    
    interface PresenceMessage extends BaseMessage {
        type: MessageType.PRESENCE;
        status: 'online' | 'away' | 'offline';
        lastSeen?: number;
    }
}
```

### Connection Management

Robust connection handling with automatic recovery:

```javascript
class WebSocketManager {
    constructor(url) {
        this.url = url;
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.heartbeatInterval = 30000;
        this.messageQueue = [];
        this.isConnected = false;
        
        this.connect();
    }
    
    connect() {
        this.ws = new WebSocket(this.url);
        
        this.ws.onopen = () => {
            this.isConnected = true;
            this.reconnectAttempts = 0;
            
            // Start heartbeat
            this.startHeartbeat();
            
            // Flush message queue
            this.flushQueue();
            
            // Send handshake
            this.send({
                type: 'handshake',
                version: '1.0.0',
                capabilities: ['typing', 'presence', 'files']
            });
        };
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleMessage(message);
        };
        
        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
        
        this.ws.onclose = () => {
            this.isConnected = false;
            this.stopHeartbeat();
            this.attemptReconnect();
        };
    }
    
    attemptReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            this.onMaxReconnectFailed();
            return;
        }
        
        this.reconnectAttempts++;
        const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
        
        console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
        
        setTimeout(() => {
            this.connect();
        }, delay);
    }
    
    startHeartbeat() {
        this.heartbeatTimer = setInterval(() => {
            if (this.isConnected) {
                this.send({ type: 'heartbeat' });
            }
        }, this.heartbeatInterval);
    }
    
    send(data) {
        const message = {
            ...data,
            id: this.generateId(),
            timestamp: Date.now()
        };
        
        if (this.isConnected && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            // Queue message for later delivery
            this.messageQueue.push(message);
        }
        
        return message.id;
    }
    
    flushQueue() {
        while (this.messageQueue.length > 0) {
            const message = this.messageQueue.shift();
            this.ws.send(JSON.stringify(message));
        }
    }
}
```

## Performance Optimization

### Rendering Performance

Techniques for maintaining 60 FPS:

```javascript
class RenderOptimizer {
    constructor() {
        this.renderQueue = [];
        this.isRendering = false;
        this.rafId = null;
        
        // Virtual DOM for diff calculations
        this.virtualDOM = {
            messages: [],
            state: {}
        };
        
        // DOM references cache
        this.domCache = new WeakMap();
    }
    
    scheduleRender(updates) {
        // Batch updates
        this.renderQueue.push(...updates);
        
        if (!this.isRendering) {
            this.isRendering = true;
            this.rafId = requestAnimationFrame(() => this.performRender());
        }
    }
    
    performRender() {
        const startTime = performance.now();
        
        // Process render queue
        const updates = this.renderQueue.splice(0, 100); // Limit batch size
        
        // Calculate minimal DOM updates
        const patches = this.calculatePatches(updates);
        
        // Apply patches
        this.applyPatches(patches);
        
        // Continue rendering if queue not empty
        if (this.renderQueue.length > 0) {
            this.rafId = requestAnimationFrame(() => this.performRender());
        } else {
            this.isRendering = false;
        }
        
        // Monitor performance
        const renderTime = performance.now() - startTime;
        if (renderTime > 16.67) {
            console.warn(`Render took ${renderTime}ms - possible jank`);
        }
    }
    
    calculatePatches(updates) {
        const patches = [];
        
        for (const update of updates) {
            const oldVNode = this.virtualDOM[update.path];
            const newVNode = update.value;
            
            const diff = this.diff(oldVNode, newVNode);
            if (diff) {
                patches.push({
                    path: update.path,
                    diff: diff
                });
            }
            
            // Update virtual DOM
            this.virtualDOM[update.path] = newVNode;
        }
        
        return patches;
    }
}
```

### Resource Loading

Progressive loading strategies:

```javascript
class ResourceLoader {
    constructor() {
        this.loadQueue = [];
        this.loadingResources = new Set();
        this.resourceCache = new Map();
        
        // Intersection Observer for lazy loading
        this.observer = new IntersectionObserver(
            (entries) => this.handleIntersection(entries),
            {
                rootMargin: '50px',
                threshold: 0.01
            }
        );
    }
    
    async loadCriticalResources() {
        // Critical CSS
        await this.loadCSS('/css/critical.css', { priority: 'high' });
        
        // Critical JavaScript
        await this.loadScript('/js/core.js', { priority: 'high' });
        
        // Preload fonts
        this.preloadFonts([
            '/fonts/inter-var.woff2',
            '/fonts/jetbrains-mono.woff2'
        ]);
        
        // Prefetch likely navigation targets
        this.prefetchResources([
            '/api/user',
            '/api/conversations/recent'
        ]);
    }
    
    async loadCSS(href, options = {}) {
        return new Promise((resolve, reject) => {
            const link = document.createElement('link');
            link.rel = 'stylesheet';
            link.href = href;
            
            if (options.priority === 'high') {
                link.fetchpriority = 'high';
            }
            
            link.onload = resolve;
            link.onerror = reject;
            
            document.head.appendChild(link);
        });
    }
    
    preloadFonts(fonts) {
        fonts.forEach(font => {
            const link = document.createElement('link');
            link.rel = 'preload';
            link.as = 'font';
            link.type = 'font/woff2';
            link.href = font;
            link.crossOrigin = 'anonymous';
            document.head.appendChild(link);
        });
    }
}
```

## Accessibility Implementation

### WCAG 2.1 AAA Compliance

Comprehensive accessibility features:

```javascript
class AccessibilityManager {
    constructor() {
        this.announcer = this.createAnnouncer();
        this.focusTrap = null;
        this.prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        
        this.initializeA11y();
    }
    
    initializeA11y() {
        // Skip navigation links
        this.addSkipLinks();
        
        // Keyboard navigation
        this.setupKeyboardNav();
        
        // Screen reader announcements
        this.setupAnnouncements();
        
        // Focus management
        this.setupFocusManagement();
        
        // High contrast mode detection
        this.detectHighContrast();
    }
    
    createAnnouncer() {
        const announcer = document.createElement('div');
        announcer.setAttribute('role', 'status');
        announcer.setAttribute('aria-live', 'polite');
        announcer.setAttribute('aria-atomic', 'true');
        announcer.className = 'sr-only';
        document.body.appendChild(announcer);
        return announcer;
    }
    
    announce(message, priority = 'polite') {
        this.announcer.setAttribute('aria-live', priority);
        this.announcer.textContent = message;
        
        // Clear after announcement
        setTimeout(() => {
            this.announcer.textContent = '';
        }, 1000);
    }
    
    setupKeyboardNav() {
        document.addEventListener('keydown', (e) => {
            // Keyboard shortcuts
            if (e.altKey) {
                switch(e.key) {
                    case '1': this.navigateTo('chat'); break;
                    case '2': this.navigateTo('drive'); break;
                    case '3': this.navigateTo('tasks'); break;
                    case '4': this.navigateTo('mail'); break;
                    case '/': this.focusSearch(); break;
                }
            }
            
            // Tab trap for modals
            if (this.focusTrap && e.key === 'Tab') {
                this.handleTabTrap(e);
            }
            
            // Escape key handling
            if (e.key === 'Escape') {
                this.handleEscape();
            }
        });
    }
}
```

## Security Considerations

### Content Security Policy

Comprehensive CSP implementation:

```html
<meta http-equiv="Content-Security-Policy" content="
    default-src 'self';
    script-src 'self' 'unsafe-inline' 'unsafe-eval';
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: blob:;
    font-src 'self' data:;
    connect-src 'self' wss://localhost:8080;
    media-src 'self';
    object-src 'none';
    frame-src 'none';
    base-uri 'self';
    form-action 'self';
    frame-ancestors 'none';
    upgrade-insecure-requests;
">
```

### XSS Prevention

Input sanitization and output encoding:

```javascript
class SecurityManager {
    sanitizeHTML(html) {
        const policy = {
            ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'a', 'br', 'p', 'code', 'pre'],
            ALLOWED_ATTR: ['href', 'title', 'target'],
            ALLOW_DATA_ATTR: false,
            RETURN_DOM: false,
            RETURN_DOM_FRAGMENT: false,
            RETURN_TRUSTED_TYPE: true
        };
        
        return DOMPurify.sanitize(html, policy);
    }
    
    escapeHTML(text) {
        const map = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#x27;',
            '/': '&#x2F;'
        };
        
        return text.replace(/[&<>"'/]/g, char => map[char]);
    }
}
```

## Summary

The gbui system provides a complete, production-ready user interface framework that delivers enterprise-grade performance while maintaining simplicity and accessibility. Through careful optimization, progressive enhancement, and standards compliance, the system achieves sub-second response times and smooth 60 FPS rendering across all supported platforms.