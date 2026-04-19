# Attendance Suite - Plano Completo

## Visão Geral

O módulo **Attendance** (Atendimento) é o sistema central de gestão de conversas humano-bot que permite transfers smooth entre o assistente IA e atendentes humanos. Integra nativamente com WhatsApp (inclui voice calls), Telegram, Teams, CRM, Marketing, Email e o motor Basic.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              ATTENDANCE SUITE                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│   │   WHATSAPP  │     │  TELEGRAM   │     │     SMS     │     │  INSTAGRAM  │   │
│   │   +Voice    │     │   +Voice    │     │             │     │             │   │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘   │
│          │                   │                   │                   │           │
│          └───────────────────┴─────────┬─────────┴───────────────────┘           │
│                                          │                                       │
│                                          ▼                                       │
│   ┌─────────────┐               ┌─────────────────┐                             │
│   │  MESSENGER  │               │  LIVEKIT + SIP  │                             │
│   └──────┬──────┘               │  Video/Audio    │                             │
│          │                       │  STT/TTS        │                             │
│          │                       └────────┬────────┘                             │
│          │                                │                                       │
│          │                                ▼                                       │
│   ┌──────┴──────┐               ┌─────────────────┐                             │
│   │    WEB      │               │  ATTENDANCE     │                             │
│   │   Chat      │──────────────►│    ENGINE       │◄────────────               │
│   └─────────────┘               └────────┬────────┘    │                          │
│                                          │             │                          │
│          ┌──────────────────────────────┼─────────────┴───────────┐              │
│          │                              │                         │              │
│          ▼                              ▼                         ▼              │
│   ┌─────────────┐          ┌─────────────────────┐     ┌────────────────────┐ │
│   │     CRM     │          │  DESTINATION CHANNELS│     │    EMAIL           │ │
│   │   MODULE    │          │  ┌────────┐ ┌───────┐│     │    MODULE          │ │
│   └─────────────┘          │  │ TEAMS │ │Google ││     └────────────────────┘ │
│                             │  │       │ │ Chat ││                            │
│   ┌─────────────┐          │  └────────┘ └───────┘│                            │
│   │  MARKETING  │          │  ┌───────┐ ┌───────┐│                            │
│   │   MODULE    │          │  │WhatsApp│ │ Web  ││                            │
│   └─────────────┘          │  │       │ │Console│                            │
│                            │  └───────┘ └───────┘│                            │
│                            └─────────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────────────┘
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              ATTENDANCE SUITE                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│   │   WHATSAPP  │     │  TELEGRAM   │     │     SMS     │     │  INSTAGRAM  │   │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘     └──────┬──────┘   │
│          │                   │                   │                   │           │
│          └───────────────────┴─────────┬─────────┴───────────────────┘           │
│                                          │                                       │
│                                          ▼                                       │
│   ┌─────────────┐               ┌─────────────────┐               ┌────────────┐│
│   │   MESSENGER │               │  LIVEKIT + SIP  │               │   TEAMS    ││
│   └──────┬──────┘               │  Video/Audio    │               └─────┬──────┘│
│          │                       │  Screen Share   │                     │       │
│   ┌──────┴──────┐               └────────┬────────┘               ┌─────┴─────┐│
│   │    SLACK     │                        │                         │    WEB    ││
│   └─────────────┘                        ▼                         └───────────┘│
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                        CHANNEL ROUTER                                   │   │
│   │  • Detecção de canal (whatsapp/telegram/sms/web/instagram/slack/teams)  │   │
│   │  • Normalização de mensagens                                            │   │
│   │  • Comandos de atendente (/queue, /take, /resolve, /video, /call)      │   │
│   └────────────────────────────────┬────────────────────────────────────────┘   │
│                                    │                                             │
│                                    ▼                                             │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │                      ATTENDANCE ENGINE                                   │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │   │
│   │  │   QUEUE     │  │  ATTENDANT   │  │    LLM      │  │   MEETING   │    │   │
│   │  │  MANAGER    │  │   MANAGER    │  │   ASSIST    │  │  (LiveKit)  │    │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │   │
│   └────────────────────────────────┬────────────────────────────────────────┘   │
│                                    │                                             │
│          ┌─────────────────────────┼─────────────────────────┐                   │
│          │                         │                         │                   │
│          ▼                         ▼                         ▼                   │
│   ┌─────────────┐          ┌─────────────┐          ┌─────────────┐             │
│   │     CRM     │          │  MARKETING  │          │    EMAIL    │             │
│   │   MODULE    │          │   MODULE    │          │   MODULE    │             │
│   └─────────────┘          └─────────────┘          └─────────────┘             │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Integração com Canais (WhatsApp/Telegram/SMS/Web/Instagram/LiveKit/SIP)

### 1.0 Arquitetura de Canais Suportados

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                              CANAIS DE ATENDIMENTO                                │
├──────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │   WHATSAPP  │  │  TELEGRAM    │  │     SMS     │  │  INSTAGRAM  │           │
│  │  (Voice)    │  │              │  │   (Twilio)  │  │   Direct    │           │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘           │
│         │                │                 │                │                    │
│         └────────────────┼─────────────────┼────────────────┘                    │
│                          │                 │                                      │
│                         ▼                 ▼    ┌──────────────────────────┐    │
│              ┌────────────────────┐        │    │   DESTINOS ATENDIMENTO   │    │
│              │ LIVEKIT + SIP      │        │    │   (Teams / Google Chat)  │    │
│              │ Video/Audio Calls │        │    │                          │    │
│              │ Screen Sharing    │────────┼────│  ┌──────────┐ ┌────────┐  │    │
│              │ Voice STT/TTS     │        │    │  │  TEAMS   │ │GOOGLE  │  │    │
│              └────────────────────┘        │    │  │          │ │ CHAT   │  │    │
│                                            │    │  └──────────┘ └────────┘  │    │
│  ┌─────────────┐                          │    └──────────────────────────┘    │
│  │  MESSENGER  │──────────────────────────┘                                      │
│  │   Facebook  │                          │                                      │
│  └─────────────┘                    ┌──────┴──────┐                             │
│                                     │ CHANNEL      │                             │
│  ┌─────────────┐                    │ ROUTER      │                             │
│  │    WEB      │─────────────────────┤              │                             │
│  │   Chat      │                    └───────────────┘                             │
│  └─────────────┘                                                                    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 1.0.1 Canais de Entrada

| Canal | Tipo | Status | Suporte a Videochamada | Voice (STT/TTS) |
|-------|------|--------|------------------------|-----------------|
| **WhatsApp** | Mensageria | ✅ Estável | ❌ Não | ✅ Sim |
| **Telegram** | Mensageria | ✅ Estável | ✅ Botões | ✅ Sim |
| **SMS** | Mensageria | ✅ Estável | ❌ Não | ❌ Não |
| **Instagram** | Mensageria | ✅ Estável | ❌ Não | ❌ Não |
| **Messenger** | Mensageria | ✅ Parcial | ❌ Não | ❌ Não |
| **Teams** | Mensageria | ✅ Parcial | ✅ Embed | ✅ Sim |
| **Web Chat** | Mensageria | ✅ Estável | ✅ LiveKit | ✅ Sim |
| **LiveKit/SIP** | Video/Audio | ✅ Estável | ✅ Completo | ✅ Completo |

### 1.0.2 Destinos de Atendimento Humano

| Destino | Descrição | Status |
|---------|-----------|--------|
| **Teams** | Atendente recebe no Microsoft Teams | ✅ Implementado |
| **Google Chat** | Atendente recebe no Google Chat | 🔜 Planejado |
| **WhatsApp** | Atendente responde via WhatsApp | ✅ Implementado |
| **Web Console** | Atendente via interface web | ✅ Implementado |

### 1.1 Arquitetura de Mensagens

O Attendance actúa como **middleware** entre os canais de entrada e o motor Basic:

```
MENSAGEM ENTRADA
       │
       ▼
┌──────────────────┐
│  CHANNEL ADAPTER │ ──► Detecta canal de origem
│  (WhatsApp/TG/   │
│   SMS/Web)       │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  NEEDS_HUMAN?    │ ──► Verifica flag na sessão
│                  │
│  • false → BASIC │ ──► Processa via motor Basic
│  • true  → ATD   │ ──► Encaminha para atendimento humano
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   RESPONSE       │ ──► Retorna resposta ao canal
│   ROUTER         │     original
└──────────────────┘
```

### 1.2 Fluxo WhatsApp

```python
# Quando cliente envia mensagem via WhatsApp:

1. WhatsAppAdapter recebe webhook
2. SessionLoader verifica needs_human:
   
   IF session.needs_human == true:
       # Routing para Attendance
       attendance_handler.process(session, message, "whatsapp")
   ELSE:
       # Routing para Basic Engine
       basic_engine.execute(session, message)

3. Se attendente responde:
   WhatsAppAdapter.send_message(attendant_response)
```

### 1.2.1 WhatsApp Voice (Chamadas de Voz)

O WhatsApp suporta **chamadas de voz** com STT (Speech-to-Text) e TTS (Text-to-Speech):

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    WHATSAPP VOICE CALL FLOW                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Cliente ──[Liga]──► WhatsApp ──[Webhook]──► BotServer                    │
│                                              │                              │
│                                              ▼                              │
│                                    ┌──────────────────┐                    │
│                                    │  Voice Handler   │                    │
│                                    │  ┌────────────┐  │                    │
│                                    │  │ STT (Whisper)│ │ ──► Texto        │
│                                    │  └────────────┘  │                    │
│                                    └────────┬─────────┘                    │
│                                             │                              │
│                                             ▼                              │
│                                    ┌──────────────────┐                    │
│                                    │  Basic Engine   │                    │
│                                    │  ou Attendance  │                    │
│                                    └────────┬─────────┘                    │
│                                             │                              │
│                                             ▼                              │
│                                    ┌──────────────────┐                    │
│                                    │  TTS (BotModels) │                    │
│                                    │  ┌────────────┐  │                    │
│                                    │  │Coqui/OpenAI│ │ ──► Áudio         │
│                                    │  └────────────┘  │                    │
│                                    └────────┬─────────┘                    │
│                                             │                              │
│                                             ▼                              │
│                              WhatsApp ◄──[Audio]── BotServer               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Configuração

```csv
name,value
whatsapp-voice-response,true
botmodels-url,http://localhost:5000
botmodels-api-key,
```

#### Componente BotModels (STT/TTS)

O sistema usa `botmodels` para processamento de voz:

```python
# botmodels/src/services/speech_service.py

class SpeechService:
    def stt(self, audio_url: str) -> str:
        # Whisper para transcrição
        # Groq como fallback rápido
        pass
    
    def tts(self, text: str, voice: str = "alloy") -> str:
        # Coqui TTS (local)
        # OpenAI TTS
        # Google Translate TTS (fallback)
        pass
```

#### Fluxo de Voz no Attendance

```
1. Cliente liga no WhatsApp
2. WhatsApp envia webhook de chamada
3. Sistema atende e inicia gravação
4. Áudio é processado via STT → Texto
5. Texto é processado:
   
   SE needs_human = true:
       → Attendente recebe transcrição
       → Attendente responde (texto ou voz)
       → Resposta → TTS → Áudio → WhatsApp
   
   SE needs_human = false:
       → Basic Engine processa
       → Resposta → TTS → Áudio → WhatsApp
```

#### Comandos de Voz

| Comando | Descrição |
|---------|-----------|
| `/voice on` | Ativar respostas de voz |
| `/voice off` | Desativar respostas de voz |
| `/call` | Solicitar chamada de volta |

#### Exemplos

```basic
' Ativar resposta de voz
SET SESSION "voice_response", true

' Desativar
SET SESSION "voice_response", false

' Verificar se é chamada de voz
IF session.call_type = "voice" THEN
    TALK "Entendi. Deixe-me verificar."
    ' Gera resposta em áudio automaticamente
END IF
```

---

## 1.3 Fluxo: Cliente Diz "Oi" no WhatsApp → Attendente

Este é o cenário mais comum. Quando um cliente inicia conversa com "Oi" no WhatsApp:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│            FLUXO: CLIENTE DIZ "OI" NO WHATSAPP                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. CLIENTE ENVIA "Oi"                                                      │
│     │                                                                       │
│     ▼                                                                       │
│  2. WHATSAPP ADAPTER RECEBE WEBHOOK                                        │
│     │                                                                       │
│     ▼                                                                       │
│  3. SESSION LOADER VERIFICA needs_human                                     │
│     │                                                                       │
│     ├─────────────────────────────┬─────────────────────────────────────┐  │
│     │                             │                                     │  │
│     ▼                             ▼                                     │  │
│  needs_human = false          needs_human = true                        │  │
│     │                             │                                     │  │
│     ▼                             ▼                                     │  │
│  BASIC ENGINE              ATTENDANCE QUEUE                               │
│  processa "Oi"              ├── Adiciona à fila                          │  │
│  (bot responde)             ├── Define priority                          │  │
│                             └── Notifica attendants (WebSocket)          │  │
│                                    │                                      │  │
│                                    ▼                                      │  │
│                             ATTENDANTE VÊ NOTIFICAÇÃO                      │  │
│                                    │                                      │  │
│                                    ▼                                      │  │
│                             /take ou clica em "Aceitar"                   │  │
│                                    │                                      │  │
│                                    ▼                                      │  │
│                             CHAT ATIVO                                     │  │
│                             └── Attendente digita resposta                 │  │
│                                    │                                      │  │
│                                    ▼                                      │  │
│                             RESPOSTA → WHATSAPP → CLIENTE                  │  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3.1 Passo a Passo Detalhado

**Passo 1:** Cliente envia "Oi" → WhatsApp API → Webhook → BotServer

**Passo 2:** Sistema verifica `needs_human`:
```rust
fn check_needs_human(session: &UserSession) -> bool {
    session.context_data.get("needs_human")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
```

**Passo 3: Se needs_human = false** → Basic Engine processa → Bot responde

**Passo 4: Se needs_human = true:**
1. Adiciona à fila de atendimento
2. Notifica attendants online (WebSocket)
3. Attendente vê notificação
4. Attendente clica "Aceitar" ou `/take`
5. Attendente digita resposta
6. Resposta → WhatsApp → Cliente

### 1.3.2 Attendant Recebe via WhatsApp

Configuração em `attendant.csv`:
```csv
id,name,channel,phone
att-001,Maria Santos,whatsapp,+5511999990001
```

Notificação:
```
📱 *Nova conversa*
De: +5511988887777 (João Silva)
Mensagem: Oi

Digite: /take para aceitar
```

Attendente responde → WhatsApp → Cliente

### 1.3.3 Attendants via Interface (Users Table)

**Não usa mais `attendant.csv`**. Usa a **tabela users** existente:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              ATTENDANTS VIA INTERFACE - users table                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    CRIAÇÃO DE FILA (UI)                             │   │
│  │  ┌─────────────────────────────────────────────────────────────┐    │   │
│  │  │ Nome da Fila: Suporte WhatsApp                               │    │   │
│  │  │ Descrição: Atendimentos via WhatsApp                        │    │   │
│  │  │                                                              │    │   │
│  │  │ Canais: ☑ WhatsApp ☐ Telegram ☐ Web ☐ Instagram           │    │   │
│  │  │                                                              │    │   │
│  │  │ Usuários (atendentes):                                     │    │   │
│  │  │   ☑ Maria Santos (maria@empresa.com)                       │    │   │
│  │  │   ☑ João Silva (joao@empresa.com)                          │    │   │
│  │  │   ☐ Ana Costa (ana@empresa.com)                            │    │   │
│  │  │                                                              │    │   │
│  │  │ [Criar Fila]                                                │    │   │
│  │  └─────────────────────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    FILAS CONFIGURADAS                               │   │
│  │  ────────────────────────────────────────────────────────────────   │   │
│  │  📋 Fila                  │ Canais        │ Atendentes │ Status      │   │
│  │  ────────────────────────────────────────────────────────────────   │   │
│  │  Suporte WhatsApp         │ WhatsApp      │ 3 ativos   │ Ativa      │   │
│  │  Vendas                  │ Web, WhatsApp │ 2 ativos   │ Ativa      │   │
│  │  Técnica                 │ Telegram      │ 1 ativo    │ Ativa      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3.4 Modelo de Dados - Filas

```sql
-- Tabela de Filas de Atendimento
CREATE TABLE attendance_queues (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    channels JSONB DEFAULT '["whatsapp"]',
    is_active BOOLEAN DEFAULT true,
    priority_order INTEGER DEFAULT 0,
    max_wait_seconds INTEGER DEFAULT 600,
    auto_assign BOOLEAN DEFAULT true,
    bot_id UUID REFERENCES bots(id),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Membros da Fila (users ↔ queue)
CREATE TABLE attendance_queue_members (
    id UUID PRIMARY KEY,
    queue_id UUID REFERENCES attendance_queues(id),
    user_id UUID REFERENCES users(id),
    is_active BOOLEAN DEFAULT true,
    max_conversations INTEGER DEFAULT 5,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 1.3.5 API de Filas

```rust
// Criar fila
POST /api/attendance/queues
{
    "name": "Suporte WhatsApp",
    "channels": ["whatsapp"],
    "user_ids": ["uuid-1", "uuid-2"]
}

// Adicionar usuário à fila
POST /api/attendance/queues/{id}/members
{"user_id": "uuid", "max_conversations": 5}
```

### 1.3.6 Atender Cliente Existente do CRM

```rust
// 1. Busca cliente no CRM
let customer = crm_contacts::table
    .filter(crm_contacts::phone.eq(phone))
    .first::<CrmContact>(conn)?;

// 2. Seleciona fila pelo canal
let queue = attendance_queues::table
    .filter(attendance_queues::channels.contains("whatsapp"))
    .filter(attendance_queues::is_active.eq(true))
    .first::<AttendanceQueue>(conn)?;

// 3. Seleciona próximo atendente (round-robin)
let member = attendance_queue_members::table
    .filter(attendance_queue_members::queue_id.eq(queue.id))
    .filter(attendance_queue_members::is_active.eq(true))
    .order(attendance_queue_members::priority.asc())
    .first::<QueueMember>(conn)?;

// 4. Associa ao usuário
let session = UserSession {
    needs_human: true,
    assigned_to: Some(member.user_id),  // ← users.id
    queue_id: Some(queue.id),
    customer_id: Some(customer.id),  // ← CRM contact
    ..
};
```

### 1.3.7 Fluxo com Cliente CRM

```
Cliente CRM existente
    │
    ▼
Envia mensagem WhatsApp
    │
    ▼
Identifica canal → fila específica
    │
    ▼
Seleciona próximo atendente (users)
    │
    ▼
Attendant vê dados do CRM:
  "João Silva - joao@email.com"
  "Cliente desde: 2022"
  "Total compras: R$ 5.000"
    │
    ▼
Responde
    │
    ▼
Ticket.assigned_to = users.id
Ticket.customer_id = crm_contacts.id
```

### 1.3.8 Console Web

```
┌─────────────────────────────────────────────────────────────────┐
│                    FILA DE ATENDIMENTO                          │
├─────────────────────────────────────────────────────────────────┤
│  🎫 #1 - Maria Santos (Você)                                    │
│     WhatsApp • João Silva (+55 11 98888-7777)                  │
│     "Oi" • 30s                                                  │
│     [Resolver] [Transferir]                                     │
│                                                                 │
│  🎫 #2 - João Silva                                             │
│     WhatsApp • Cliente Novo                                     │
│     "Preciso de ajuda" • 2min                                  │
│     [Aceitar]                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3.5 WebSocket Notificação

```json
{
  "type": "new_conversation",
  "session_id": "abc-123",
  "channel": "whatsapp",
  "customer": {"name": "João Silva", "phone": "+5511988887777"},
  "message": "Oi"
}
```

---

## 1.4 Fluxo Telegram

Mesma lógica do WhatsApp, com comandos específicos:

```
/start - Iniciar conversa
/agent - Solicitar atendente humano
/queue - Ver fila (atendente)
/resolve - Encerrar atendimento (atendente)
```

### 1.4 Fluxo SMS

```
SMS recebido → Normalizar → Verificar needs_human →
  → Se true: Attendance (com limite de 160 chars)
  → Se false: Basic Engine
```

### 1.5 Modo Bypass (Midleman)

O Attendance pode actuar como **midleman puro** (sem IA):

```
┌────────────┐     ┌────────────┐     ┌────────────┐
│  CLIENTE   │────►│   BOT      │────►│ ATENDENTE  │
│  (WhatsApp)│     │ (Attendance│     │  HUMANO    │
│            │◄────│   Bypass)  │◄────│            │
└────────────┘     └────────────┘     └────────────┘
```

**Configuração:**
```csv
name,value
attendance-bypass-mode,true
attendance-auto-transfer,true
attendance-transfer-keywords,human,atendente,pessoa,atendimento
```

### 1.6 Transferência para Teams

O attendance pode enviar a conversa para **Microsoft Teams** onde o atendente recebe a mensagem:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                  TRANSFER TO TEAMS FLOW                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Cliente        Bot                     Attendance      Microsoft Teams      │
│   (WhatsApp)                                                       (Atendente)  │
│       │             │                         │                  │           │
│       │────────────►│                         │                  │           │
│       │  Mensagem  │                         │                  │           │
│       │             │                         │                  │           │
│       │  (precisa  │                         │                  │           │
│       │   humano)  │                         │                  │           │
│       │             │                         │                  │           │
│       │             ├────────────────────────►│                  │           │
│       │             │  TRANSFER TO HUMAN      │                  │           │
│       │             │  destination=teams      │                  │           │
│       │             │                         │                  │           │
│       │             │                         ├─────────────────►│           │
│       │             │                         │  Mensagem Teams │           │
│       │             │                         │                  │           │
│       │◄────────────┤◄────────────────────────┤  Resposta       │           │
│       │  Resposta  │   (forwarded back)      │                  │           │
│       │             │                         │                  │           │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Configuração Teams

```csv
name,value
teams-enabled,true
teams-app-id,
teams-app-password,
teams-tenant-id,
teams-bot-id,
attendance-default-destination,teams
```

#### Transferir para Teams

```basic
' Transferir para Teams
TRANSFER TO HUMAN "support", "normal", "Cliente precisa de ajuda", "teams"

' Ou especificar o destino
result = TRANSFER TO HUMAN({
    department: "support",
    destination: "teams"
})
```

#### Comandos no Teams

O atendente pode usar comandos no Teams:

```
/resolve - Encerrar atendimento
/transfer @nome - Transferir para outro atendente
/queue - Ver fila
/context - Ver contexto do cliente
```

### 1.7 Transferência para Google Chat

Planejado para futuras implementações:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│              GOOGLE CHAT DESTINATION (PLANEJADO)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Cliente ──► WhatsApp ──► Bot ──► Attendance ──► Google Chat ──► Atendente │
│                                                                           │
│   Configuração futura:                                                     │
│   name,value                                                              │
│   google-chat-enabled,true                                                │
│   google-chat-bot-token,                                                  │
│   google-chat-space-id,                                                   │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.6.1 Teams Voice Calls

O Teams suporta chamadas de voz e vídeo diretamente:

```csv
name,value
teams-voice-enabled,true
teams-meeting-enabled,true
```

```basic
' Criar reunião Teams para atendimento
result = CREATE MEETING({
    "type": "teams",
    "title": "Suporte - " + customer.name,
    "participants": [customer.email]
})

TALK "Vou iniciar uma reunião Teams com você."
TALK result.join_url
```

---

## 1.10 Instagram Direct

### 1.10.1 Configuração

```csv
name,value
instagram-enabled,true
instagram-access-token,
instagram-app-secret,
instagram-webhook-verify-token,
```

### 1.10.2 Fluxo

```
Instagram User → Instagram API → Webhook → Channel Router → Attendance
                                                                    ↓
                                              needs_human=true → Fila de Atendimento
                                                                    ↓
                                              Atendente responde → Instagram API → User
```

### 1.10.3 Limitações do Instagram

| Recurso | Suporte | Observação |
|---------|---------|-------------|
| Texto | ✅ | Suportado |
| Imagens | ✅ | Download e reenvio |
| Vídeos | ✅ | Download e reenvio |
| Áudio | ⚠️ | Limitado |
| Videochamada | ❌ | Não disponível na API |
| Compartilhamento de tela | ❌ | Não disponível |

### 1.10.4 Workaround para Videochamada

Quando cliente Instagram precisa de videochamada:

```basic
' Instagram não suporta videochat nativo
' Ofereça alternativas:

TALK "Para melhor atendê-lo, gostaria de fazer uma videochamada?"
TALK "Posso criar uma sala de reunião agora. Clique no link:"
TALK meeting_link

' Attendente cria reunião via comando
' /video ou /call
```

---

## 1.11 LiveKit + SIP (Videochamadas)

### 1.11.1 Arquitetura LiveKit

O sistema já possui integração com LiveKit para videochamadas:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          LIVEKIT INTEGRATION                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────────────────┐  │
│  │  Attendance │───►│ Meeting      │───►│ LiveKit Room                 │  │
│  │   Queue     │    │   Service    │    │ ┌─────────────────────────┐ │  │
│  └─────────────┘    └──────────────┘    │ │ • Video (WebRTC)         │ │  │
│                                           │ │ • Audio                  │ │  │
│  ┌─────────────┐    ┌──────────────┐    │ │ • Screen Sharing         │ │  │
│  │  Atendente  │───►│ Token        │───►│ │ • Transcription (AI)     │ │  │
│  │  Browser    │    │   Generator  │    │ │ • Recording              │ │  │
│  └─────────────┘    └──────────────┘    │ │ • Whiteboard             │ │  │
│                                           │ └─────────────────────────┘ │  │
│  ┌─────────────┐    ┌──────────────┐    │ │                           │  │
│  │   Cliente   │───►│ Join URL     │───►│ │ SIP Gateway (futuro)     │  │
│  │  Browser    │    │              │    │ │ • PSTN inbound           │  │
│  └─────────────┘    └──────────────┘    │ │ • PSTN outbound          │  │
│                                           │ │ • SIP trunk              │  │
│                                           └───────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.11.2 Configuração LiveKit

```csv
name,value

# LiveKit Core
livekit-url,wss://livekit.yourserver.com
livekit-api-key,
livekit-api-secret,
livekit-room-prefix,attendance-

# SIP Configuration (futuro)
sip-enabled,false
sip-trunk-name,
sip-phone-number,
sip-inbound-route,
sip-outbound-route,

# Recording
livekit-recording-enabled,true
livekit-storage-bucket,recordings

# Transcription
livekit-transcription-enabled,true
transcription-language,pt-BR
```

### 1.11.3 Iniciar Videochamada no Attendance

**Comando do Atendente:**

```
/video - Iniciar videochamada
/video link - Gerar link para cliente
/video invite @cliente - Convidar para sala ativa
/video end - Encerrar videochamada
```

**Comando BASIC:**

```basic
' Criar sala de reunião para atendimento
result = CREATE MEETING({
    "title": "Atendimento - " + customer.name,
    "type": "support",
    "expires_in": 3600,
    "max_participants": 2,
    "recording": false,
    "transcription": true
})

IF result.success THEN
    SET SESSION "meeting_room", result.room_id
    SET SESSION "meeting_url", result.join_url
    
    TALK "Vou iniciar uma videochamada para melhor atendê-lo."
    TALK result.join_url
    
    ' Notifica atendente
    NOTIFY attendant, "Cliente entrou na sala: " + result.join_url
END IF
```

### 1.11.4 Compartilhamento de Tela

**Durante videochamada:**

```basic
' Atendente pode compartilhar tela
' Cliente pode compartilhar tela

' Detectar compartilhamento
IF meeting.participant.shared_screen THEN
    TALK "Cliente está compartilhando a tela"
END IF

' Solicitar compartilhamento
meeting.request_screen_share(participant_id)
```

### 1.11.5 Fluxo de Videochamada no Attendance

```
1. Cliente entra em contato (qualquer canal)
2. Atendente aceita o atendimento
3. Atendente decide fazer videochamada:
   /video

4. Sistema cria sala LiveKit
5. Sistema gera link de acesso
6. Envia link para cliente (mesmo canal ou email)

7. Cliente clica no link
8. Navegador abre → Permissões de câmera/microfone
9. Entra na sala de videochamada

10. Ambos (atendente + cliente) podem:
    • Ver vídeo
    • Ouvir áudio
    • Compartilhar tela
    • Ver transcrição ao vivo
    • Usar whiteboard

11. /resolve → Encerrar atendimento
12. Sala é fechada ou arquivada
```

### 1.11.6 API de Meeting

```rust
// Endpoints disponíveis em botserver/src/meet/mod.rs

POST /api/meet/create           // Criar sala
GET  /api/meet/rooms            // Listar salas
GET  /api/meet/rooms/{id}       // Obter sala
POST /api/meet/rooms/{id}/join  // Entrar na sala
POST /api/meet/token            // Gerar token
POST /api/meet/transcription    // Iniciar transcrição
POST /api/meet/invite           // Enviar convite

// WebSocket
WS  /api/meet/ws               // WebSocket de meeting

// Conversations
POST /api/meet/conversations/create
POST /api/meet/conversations/{id}/join
POST /api/meet/conversations/{id}/calls/start
```

---

## 1.12 SIP Gateway (Futuro)

### 1.12.1 Arquitetura SIP

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SIP GATEWAY (PLANEJADO)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   PSTN Network                                                             │
│        │                                                                    │
│        ▼                                                                    │
│  ┌─────────────┐      ┌──────────────┐      ┌─────────────────────────┐   │
│  │   SIP       │─────►│   LiveKit    │─────►│   Attendance           │   │
│  │   Trunk     │      │   Gateway    │      │   Queue                │   │
│  └─────────────┘      └──────────────┘      └─────────────────────────┘   │
│                              │                                              │
│                     ┌────────┴────────┐                                    │
│                     │                 │                                    │
│                     ▼                 ▼                                    │
│              ┌────────────┐    ┌────────────┐                            │
│              │   Inbound  │    │  Outbound  │                            │
│              │   Calls    │    │   Calls    │                            │
│              └────────────┘    └────────────┘                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.12.2 Casos de Uso SIP

| Cenário | Descrição |
|---------|-----------|
| **Inbound** | Cliente liga para número fixo → Direcionado para fila de atendimento |
| **Outbound** | Atendente faz ligação para cliente → ID do atendimento |
| **Callback** | Cliente agenda retorno → Sistema liga na hora marcada |
| **IVR** | Menu de opções antes de entrar na fila |

### 1.12.3 Comandos SIP

```
/call - Iniciar ligação
/call back - Ligar de volta
/callback 5511999999999 - Agendar retorno
/hangup - Desligar
/hold - Colocar em espera
/transfer - Transferir ligação
```

---

## 2. Integração com CRM Module

### 2.1 Dados do Cliente em Tempo Real

O Attendance partilha dados com CRM para contexto do atendente:

```rust
// attendance/queue.rs - dados do cliente disponíveis
struct QueueItem {
    session_id: Uuid,
    user_id: Uuid,
    bot_id: Uuid,
    channel: String,
    // Campos CRM
    user_name: String,
    user_email: Option<String>,
    user_phone: Option<String>,
    // Contexto adicional
    last_message: String,
    priority: i32,
    assigned_to: Option<Uuid>,
}
```

### 2.2 Busca Automática de Dados CRM

```basic
' Quando transfer para humano, busca dados do CRM
TRANSFER TO HUMAN "support"

' O sistema automaticamente busca:
customer = FIND "customers", "phone='" + session.phone + "'"
IF customer FOUND THEN
    SET SESSION "customer_name", customer.name
    SET SESSION "customer_tier", customer.tier
    SET SESSION "customer_lifetime_value", customer.ltv
END IF
```

### 2.3 Campos CRM Disponíveis

| Campo | Descrição | Exemplo |
|-------|-----------|---------|
| `customer_id` | ID único do cliente | `cust-001` |
| `name` | Nome completo | `João Silva` |
| `email` | Email | `joao@email.com` |
| `phone` | Telefone | `+5511999999999` |
| `tier` | Tier do cliente | `premium`, `gold`, `standard` |
| `ltv` | Lifetime Value | `15000.00` |
| `last_purchase` | Última compra | `2024-01-15` |
| `tags` | Tags do cliente | `vip,dev,nps-9` |

### 2.4 Automação CRM via Attendance

```basic
' Regra: Se cliente premium, transfere com alta prioridade
customer = FIND "customers", "phone='" + session.phone + "'"

IF customer.tier = "premium" THEN
    TRANSFER TO HUMAN "vip-support", "high", "Cliente premium"
ELSE
    TRANSFER TO HUMAN "support"
END IF
```

### 2.5 Logging de Atendimento

```csv
name,value
attendance-crm-logging,true
attendance-log-fields,session_id,customer_id,attendant_id,start_time,end_time,sentiment
```

---

## 3. Integração com Marketing Module

### 3.1 Campanhas de Proactive Outreach

O Attendance pode iniciar conversas via Marketing:

```python
# Marketing module.trigger_attendance()
# Envia mensagem proativa e marca needs_human=true

MENSAGEM: "Olá João! Temos uma oferta especial para você."
           "[Atendente disponível para conversar]"

SET SESSION "needs_human", true
SET SESSION "campaign_id", "summer-sale-2024"
SET SESSION "lead_source", "marketing-campaign"
```

### 3.2 Dados de Campanha em Attendance

| Campo | Descrição |
|-------|-----------|
| `campaign_id` | ID da campanha营销 |
| `campaign_name` | Nome da campanha |
| `utm_source` | Fonte UTM |
| `utm_medium` | Medio UTM |
| `ad_id` | ID do anúncio |
| `segment` | Segmento do lead |

### 3.3 Qualification de Leads

```basic
' Após atendimento, marca lead como qualificationado
IF attendant.resolved THEN
    customer = FIND "customers", "phone='" + session.phone + "'"
    
    IF customer NOT FOUND THEN
        ' Cria novo lead
        CREATE "leads", {
            "name": session.user_name,
            "phone": session.phone,
            "source": session.lead_source,
            "status": "contacted",
            "attendant_id": attendant.id,
            "notes": conversation.summary
        }
    ELSE
        ' Atualiza existente
        UPDATE "customers", customer.id, {
            "status": "qualified",
            "last_contact": NOW(),
            "attendant_id": attendant.id
        }
    END IF
END IF
```

---

## 4. Integração com Email Module

### 4.1 Notifications por Email

O Attendance pode enviar emails de notificação:

```csv
name,value
attendance-email-notify,true
attendance-email-template,attendant-assignment
attendance-email-recipient,attendant
attendance-email-bcc,supervisor@empresa.com
```

### 4.2 Tipos de Notificação

| Tipo | Quando | Destinatário |
|------|--------|--------------|
| `new_assignment` | Nova conversa atribuída | Atendente |
| `queue_alert` | Fila > 10 conversas | Supervisor |
| `customer_waiting` | Cliente aguardando > 5min | Atendente |
| `sla_breach` | SLA violado | Gerente |
| `resolved` | Atendimento encerrado | Cliente (opcional) |

### 4.3 Email como Canal de Resposta

```python
# Se cliente não está no WhatsApp, pode responder por email

IF channel = "email" THEN
    # Renderiza template de resposta
    response = EMAIL.render(
        template="attendant-response",
        data={
            "customer_name": session.user_name,
            "message": attendant.message,
            "attendant_name": attendant.name,
            "company": config.company_name
        }
    )
    
    EMAIL.send(
        to=customer.email,
        subject=f"Resposta: {original_subject}",
        body=response
    )
END IF
```

---

## 5. Integração com Bot e Basic Engine

### 5.1 Palavra-chave TRANSFER TO HUMAN

```basic
' Transferência simples
TRANSFER TO HUMAN

' Transferência com destino específico
TRANSFER TO HUMAN "João Silva"
TRANSFER TO HUMAN "suporte técnico"
TRANSFER TO HUMAN "vendas", "high"

' Transferência com contexto
TRANSFER TO HUMAN "suporte", "normal", "Cliente com problema no pagamento"
```

### 5.2 Estados da Sessão

```rust
struct UserSession {
    id: Uuid,
    bot_id: Uuid,
    user_id: Uuid,
    // Flag principal de attendance
    needs_human: bool,
    
    // Dados do attendance
    context_data: HashMap<String, Value> {
        "attendant_id": "att-001",
        "attendant_name": "Maria",
        "queue_position": 3,
        "transfer_reason": "Problema técnico",
        "transfer_time": "2024-01-15T10:30:00Z",
    }
}
```

### 5.3 Palavras-chave Related

```basic
' Checar se precisa de humano
IF session.needs_human THEN
    TALK "Você está em atendimento humano."
END IF

' Obter posição na fila
position = session.queue_position

' Obter atendente atual
attendant = session.attendant_id

' Retornar para bot (apenas atendente)
SET SESSION "needs_human", false
```

### 5.4 API REST de Attendance

```python
# Endpoints disponíveis

GET  /api/attendance/queue                 # Lista fila
GET  /api/attendance/attendants            # Lista atendentes
POST /api/attendance/assign                # Atribui conversa
POST /api/attendance/transfer              # Transfere entre atendentes
POST /api/attendance/resolve/<session_id>  # Resolve atendimento
GET  /api/attendance/insights              # Métricas

# Endpoints LLM Assist
POST /api/attendance/llm/tips              # Dicas IA
POST /api/attendance/llm/polish             # Polir mensagem
POST /api/attendance/llm/smart-replies     # Respostas sugeridas
GET  /api/attendance/llm/summary/<id>      # Resumo conversa
POST /api/attendance/llm/sentiment         # Análise sentimento
```

---

## 6. Arquitetura de Filas e Atendentes

### 6.1 Estrutura de Dados

```rust
// Queue Item - Item na fila de atendimento
struct QueueItem {
    session_id: Uuid,
    user_id: Uuid,
    bot_id: Uuid,
    channel: String,          // whatsapp, telegram, sms, web
    user_name: String,
    user_email: Option<String>,
    last_message: String,
    waiting_time_seconds: i64,
    priority: i32,          // 0=low, 1=normal, 2=high, 3=urgent
    status: QueueStatus,    // waiting, assigned, active, resolved
    assigned_to: Option<Uuid>,
    assigned_to_name: Option<String>,
}

// Attendant - Atendente humano
struct Attendant {
    id: String,             // att-001
    name: String,
    channel: String,        // all, whatsapp, telegram, web
    preferences: String,    // sales, support, technical
    department: Option<String>,
    status: AttendantStatus, // online, busy, away, offline
    active_conversations: i32,
}
```

### 6.2 Status de Atendentes

| Status | Descrição | Recebe Novas Conversas? |
|--------|-----------|------------------------|
| `online` | Disponível | ✅ Sim |
| `busy` | Em atendimento | ❌ Não |
| `away` | Temporariamente indisponível | ❌ Não |
| `offline` |离线 | ❌ Não |

### 6.3 Prioridades de Conversa

| Prioridade | Valor | Uso |
|------------|-------|-----|
| `low` | 0 | Consultas gerais |
| `normal` | 1 | Padrão |
| `high` | 2 | Clientes VIP, tempo-sensível |
| `urgent` | 3 | Escalações, reclamações |

### 6.4 Routing Inteligente

```python
def route_to_attendant(session, attendants):
    # 1. Filtra por canal
    eligible = [a for a in attendants 
                if a.channel in ["all", session.channel]]
    
    # 2. Filtra por status
    eligible = [a for a in eligible if a.status == "online"]
    
    # 3. Ordena por carga de trabalho
    eligible.sort(key=lambda a: a.active_conversations)
    
    # 4. Aplica preferências
    if session.topic:
        preferred = [a for a in eligible 
                    if a.preferences == session.topic]
        if preferred:
            return preferred[0]
    
    # 5. Retorna menor carga
    return eligible[0] if eligible else None
```

---

## 7. Módulo LLM Assist

### 7.1 Funcionalidades

| Funcionalidade | Descrição | Comando WhatsApp |
|----------------|-----------|------------------|
| `tips` | Dicas para o atendente | `/tips` |
| `polish` | Polir mensagem antes de enviar | `/polish <msg>` |
| `smart-replies` | Respostas sugeridas | `/replies` |
| `summary` | Resumo da conversa | `/summary` |
| `sentiment` | Análise de sentimento | Automático |

### 7.2 Exemplo de Uso

```
Cliente: Preciso.cancelar meu pedido

Atendente: /tips
Bot: 💡 Dicas:
    • Cliente quer cancelar pedido
    • Pergunte o número do pedido
    • Verifique política de cancelamento

Atendente: /polish Gostaria de me ajudar com o cancelamento
Bot: ✨ Polido:
    "Olá! Ficarei feliz em ajudá-lo com o cancelamento."

Atendente: Olá! Ficarei feliz em ajudá-lo com o cancelamento.
[Enviado para cliente]
```

---

## 8. Configuração Completa

### 8.1 config.csv

```csv
name,value

# === ATENDIMENTO BÁSICO ===
crm-enabled,true
attendance-enabled,true

# === FILA ===
attendance-queue-size,50
attendance-max-wait-seconds,600
attendance-priority-default,normal

# === ATENDENTES ===
attendance-auto-assign,true
attendance-slack-webhook,

# === CANAIS ===
attendance-whatsapp-commands,true
attendance-telegram-commands,true

# === BYPASS MODE ===
attendance-bypass-mode,false
attendance-auto-transfer,false
attendance-transfer-keywords,human,atendente,pessoa,falar com

# === LLM ASSIST ===
attendant-llm-tips,true
attendant-polish-message,true
attendant-smart-replies,true
attendant-auto-summary,true
attendant-sentiment-analysis,true

# === CRM INTEGRATION ===
attendance-crm-logging,true
attendance-customer-fields,name,email,phone,tier,ltv

# === EMAIL NOTIFICATIONS ===
attendance-email-notify,false
attendance-email-template,attendant-response
```

### 8.2 attendant.csv

```csv
id,name,channel,preferences,department,aliases,phone,email
att-001,Maria Santos,all,sales,commercial,maria;mari,5511999990001,maria@empresa.com
att-002,João Silva,whatsapp;web,support,support,joao;js,5511999990002,joao@empresa.com
att-003,Ana Costa,telegram,technical,engineering,ana;anc,5511999990003,ana@empresa.com
att-004,Pedro Oliveira,all,collections,finance,pedro;po,5511999990004,pedro@empresa.com
```

---

## 9. Fluxos de Conversa

### 9.1 Fluxo 1: Cliente Solicita Humano

```
Cliente: Quero falar com uma pessoa
    │
    ▼
Bot detecta keyword "pessoa"
    │
    ▼
TRANSFER TO HUMAN
    │
    ├──► needs_human = true
    ├──► Adiciona à fila
    ├──► Notifica atendentes (WebSocket)
    └──► Envia "Aguarde, transferir para atendente..."
    │
    ▼
Atendente recebe notificação
    │
    ▼
Atendente aceita /take
    │
    ▼
Chat entre cliente e atendente
    │
    ▼
Atendente /resolve
    │
    ├──► needs_human = false
    └──► Volta para Bot
```

### 9.2 Fluxo 2: Bot Transfere Automaticamente

```
Cliente: Não consigo acessar minha conta
    │
    ▼
Bot tenta resolver (3 tentativas)
    │
    ├──► Falha → Analisa sentimento
    │
    ▼
IF sentiment.score < -0.5 OR intent = "escalate" THEN
    │
    ▼
    TRANSFER TO HUMAN "support", "high", "Tentativas=3, Sentimento=negativo"
```

### 9.3 Fluxo 3: Bypass Mode (Midleman)

```
Cliente: (mensagem WhatsApp)
    │
    ▼
Attendance detecta:
    needs_human = true (via config bypass)
    attendance-bypass-mode = true
    │
    ▼
SEM passar pelo Basic Engine
    │
    ├──► Direto para fila
    └──► Notifica atendentes
    │
    ▼
Atendente responde
    │
    ▼
Response enviada diretamente para WhatsApp
```

### 9.4 Fluxo 4: Videochamada (LiveKit)

```
Cliente: Preciso de ajuda com problema técnico
    │
    ▼
Bot tenta resolver (3 tentativas)
    │
    ▼
IF complexidade > threshold THEN
    │
    ▼
    TRANSFER TO HUMAN "suporte técnico"
    │
    ▼
Atendente aceita
    │
    ▼
Atendente: /video
    │
    ├──► Cria sala LiveKit
    ├──► Gera link de acesso
    └──► Envia link para cliente
    │
    ▼
Cliente acessa link
    │
    ├──► Pede permissão câmera/mic
    ├──► Entra na sala
    └──► Vídeochat dimulai
    │
    ├──► Compartilhamento de tela
    ├──► Whiteboard
    └──► Transcrição em tempo real
    │
    ▼
/resolve → Sala encerrada
    │
    ├──► Gravação disponível (se enabled)
    ├──► Transcrição salva
    └──► Retorna para Bot
```

### 9.5 Fluxo 5: Videochamada Direta (cliente inicia)

```
Cliente: (do WhatsApp)
Quero fazer videochamada
    
    │
    ▼
Bot detecta intent = "video_call"
    │
    ▼
TALK "Vou criar uma sala de videochamada para você."
    
    │
    ▼
CREATE MEETING({type: "support"})
    │
    ▼
TALK "Clique no link para entrar: " + meeting_url
    
    │
    ▼
Atendente já está na sala esperando
    │
    ▼
Cliente entra → Videochamada inicia
```

---

## 10. Métricas e Analytics

### 10.1 KPIs de Atendimento

| KPI | Descrição | Meta |
|-----|-----------|------|
| `avg_wait_time` | Tempo médio de espera | < 60s |
| `first_response_time` | Tempo até 1ª resposta | < 30s |
| `resolution_rate` | Taxa de resolução | > 85% |
| `customer_satisfaction` | NPS pós-atendimento | > 7 |
| `attendant_utilization` | Utilização dos atendentes | > 70% |
| `transfers_rate` | Taxa de transferência | < 20% |

### 10.1.1 KPIs de Videochamada

| KPI | Descrição | Meta |
|-----|-----------|------|
| `video_call_requests` | Solicitações de videochamada | - |
| `video_calls_completed` | Videochamadas completadas | > 80% |
| `avg_video_duration` | Duração média de videochamadas | < 15min |
| `screen_share_usage` | Uso de compartilhamento de tela | > 40% |
| `transcription_accuracy` | Acurácia da transcrição | > 90% |

### 10.2 Dashboard

```
┌────────────────────────────────────────────┐
│         ATTENDANCE DASHBOARD               │
├────────────────────────────────────────────┤
│                                            │
│  FILA: 5 │ ATENDENTES: 8/10 │ ONLINE: 6    │
│                                            │
│  ┌─────────────┐ ┌─────────────┐          │
│  │ TEMPO MÉDIO │ │ RESOLUÇÃO   │          │
│  │   45s       │ │   92%       │          │
│  └─────────────┘ └─────────────┘          │
│                                            │
│  POR CANAL:                                │
│  WhatsApp  ████████████ 65%                │
│  Web       ██████ 25%                      │
│  Telegram  ██ 10%                          │
│                                            │
└────────────────────────────────────────────┘
```

---

## 11. Casos de Uso

### 11.1 E-commerce - Suporte

1. Cliente pergunta sobre pedido
2. Bot tenta resolver com informações do pedido
3. Se não conseguir após 3 tentativas → TRANSFER TO HUMAN "suporte"
4. Atendente recebe contexto completo (pedido, cliente)
5. Atendente resolve → /resolve
6. Sistema cria/atualiza ticket no CRM

### 11.2 Vendas - Qualificação

1. Lead entra via WhatsApp (campanha)
2. Bot faz qualificação inicial
3. Se lead = "quente" → TRANSFER TO HUMAN "vendas", "high"
4. Atendente de vendas recebe com dados do lead
5. Atendente fecha venda → /resolve
6. Sistema cria oportunidade no CRM

### 11.3 Cobrança - Negociação

1. Cliente em atraso recebe mensagem proativa
2. Se cliente responde → needs_human = true
3. Atendente de cobrança recebe
4. Negocia dívida → registra no CRM
5. /resolve → cliente volta para fluxo de cobrança

### 11.4 Suporte Técnico - Escalação

1. Cliente reporta problema técnico
2. Bot tenta solução básica
3. Se complexidade > threshold → TRANSFER TO HUMAN "técnico"
4. Atendente técnico com acesso a sistema
5. Resolve ou escala para equipe de TI

---

## 12. Troubleshooting

### 12.1 Problemas Comuns

| Problema | Causa | Solução |
|----------|-------|---------|
| Mensagem não vai para atendente | `crm-enabled=false` | Ativar em config.csv |
| Atendente não recebe notificação | Status != online | Verificar attendant.csv |
| Transfer não encontra ninguém | Nenhum atendente online | Configurar horário ou fallback |
| Cliente preso em modo humano | /resolve não executado | Executar manualmente |
| WhatsApp não entrega resposta | Phone inválido | Verificar país + número |

### 12.2 Problemas de Videochamada

| Problema | Causa | Solução |
|----------|-------|---------|
| Link de videochamada não funciona | Sala expirada | Gerar novo link |
| Cliente sem câmera/mic | Permissão negada | Orientar cliente |
| Videochamada trava | Rede instável | Reduzir qualidade |
| Transcrição não funciona | API key inválida | Verificar config |
| Gravação não inicia | Storage cheio | Limpar espaço |

### 12.3 Debug

```bash
# Ver fila de atendimento
GET /api/attendance/queue

# Ver atendentes
GET /api/attendance/attendants

# Ver sessão específica
GET /api/session/<session_id>

# Logs de attendance
grep "attendance" botserver.log
```

---

## 13. Evolução Futura

### 13.1 Features Planejadas

- [ ] **Multi-tenant** - Múltiplas empresas
- [ ] **Skills-based routing** - Routing por habilidade
- [ ] **SLA alerts** - Alertas de SLA
- [ ] **Chatbot cobros** - Chatbot para cobrança
- [ ] **Video call** - ✅ Implementado (LiveKit)
- [ ] **Screen sharing** - ✅ Implementado
- [ ] **Co-browse** - Compartilhamento de tela
- [ ] **Knowledge base** - Base de conhecimento
- [ ] **Canned responses** - Respostas pré-definidas

### 13.2 Integrações Atuais e Futuras

#### Canais de Entrada (Implementados)

| Canal | Status | Voice (STT/TTS) |
|-------|--------|-----------------|
| WhatsApp | ✅ Estável | ✅ Implementado |
| Telegram | ✅ Estável | ✅ Implementado |
| Instagram | ✅ Parcial | ❌ Não |
| Messenger | ✅ Parcial | ❌ Não |
| Teams | ✅ Parcial | ✅ Implementado |
| Web Chat | ✅ Estável | ✅ Implementado |
| SMS | ✅ Estável | ❌ Não |
| LiveKit/SIP | ✅ Estável | ✅ Completo |

#### Destinos de Atendimento Humano

| Destino | Status | Descrição |
|---------|--------|-----------|
| **Teams** | ✅ Implementado | Atendente recebe no MS Teams |
| **Google Chat** | 🔜 Planejado | Atendente recebe no Google Chat |
| **WhatsApp** | ✅ Implementado | Atendente responde via WA |
| **Web Console** | ✅ Implementado | Interface web |

#### Features Planejadas

- [ ] **Multi-tenant** - Múltiplas empresas
- [ ] **Skills-based routing** - Routing por habilidade
- [ ] **SLA alerts** - Alertas de SLA
- [ ] **Chatbot cobros** - Chatbot para cobrança
- [x] **Video call** - ✅ Implementado (LiveKit)
- [x] **Screen sharing** - ✅ Implementado
- [x] **WhatsApp Voice** - ✅ Implementado (STT/TTS)
- [x] **Teams Voice** - ✅ Implementado
- [ ] **Co-browse** - Compartilhamento de tela
- [ ] **Knowledge base** - Base de conhecimento
- [ ] **Canned responses** - Respostas pré-definidas
- [ ] **SIP Gateway** - Planejado
- [ ] **PSTN Calls** - Planejado

---

## 15. Kanban View para Fila de Atendimento

### 15.1 Visão Geral

O Kanban é uma view visual para gerenciar a fila de atendimento, permitindo arrastar cards entre colunas.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KANBAN - FILA DE ATENDIMENTO                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   NOVOS     │  │  EM ATEND.  │  │  AGUARDANDO │  │  RESOLVIDOS │       │
│  │  (New)      │  │  (Active)   │  │  (Pending)  │  │  (Done)     │       │
│  ├─────────────┤  ├─────────────┤  ├─────────────┤  ├─────────────┤       │
│  │ ┌─────────┐ │  │ ┌─────────┐ │  │ ┌─────────┐ │  │ ┌─────────┐ │       │
│  │ │ Card #1 │ │  │ │ Card #3 │ │  │ │ Card #5 │ │  │ │ Card #7 │ │       │
│  │ │ João    │ │  │ │ Maria   │ │  │ │ Ana     │ │  │ │ Resolv. │ │       │
│  │ │ WhatsApp │ │  │ │ WhatsApp│ │  │ │ Telegram│ │  │ │ 15min   │ │       │
│  │ └────┬────┘ │  │ └────┬────┘ │  │ └────┬────┘ │  │ └─────────┘ │       │
│  │      ▼      │  │      │      │  │      ▼      │  │             │       │
│  │ ┌─────────┐ │  │      └──────┼──►│             │  │             │       │
│  │ │ Card #2 │ │  │             │  │             │  │             │       │
│  │ │ Carlos  │ │  │             │  │             │  │             │       │
│  │ │ Instagram│ │  │             │  │             │  │             │       │
│  │ └─────────┘ │  │             │  │             │  │             │       │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
│ drag & drop → mover cards entre colunas                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 15.2 Colunas do Kanban

| Coluna | Status | Descrição |
|--------|--------|-----------|
| `new` | Novos | Clientes aguardando primeiro atendimento |
| `active` | Em Atendimento | Já aceitos por attendant |
| `pending` | Aguardando | Cliente não respondeu |
| `resolved` | Resolvidos | Atendimento concluído |

### 15.3 Estrutura do Card

```
┌────────────────────────────────────────┐
│ #ID - João Silva                    │
│ ─────────────────────────────────────  │
│ 📱 WhatsApp • +55 11 98888-7777       │
│ 💬 "Preciso de ajuda com meu pedido"  │
│ ─────────────────────────────────────  │
│ ⏱️ 5min │ Prioridade: Alta │ Att: Maria │
│ Tags: [vip] [pedido]                   │
└────────────────────────────────────────┘
```

### 15.4 Implementação

#### API Endpoints

```rust
// GET - Listar com grouping por status
GET /api/attendance/kanban?bot_id={id}

// PUT - Mover card entre colunas
PUT /api/attendance/kanban/move
{
    "session_id": "uuid",
    "from_status": "new",
    "to_status": "active"
}
```

#### Frontend (attendant.js)

```javascript
// Renderizar Kanban
function renderKanban(queueItems) {
    const columns = {
        new: queueItems.filter(i => i.status === 'waiting'),
        active: queueItems.filter(i => i.status === 'active'),
        pending: queueItems.filter(i => i.status === 'pending'),
        resolved: queueItems.filter(i => i.status === 'resolved')
    };
    
    columns.forEach((items, status) => {
        renderColumn(status, items);
    });
}

// Drag & Drop
function setupDragDrop() {
    document.querySelectorAll('.kanban-card').forEach(card => {
        card.draggable = true;
        card.addEventListener('dragend', handleDragEnd);
    });
}
```

---

## 16. Tickets (Issues) Integrados ao Atendimento

### 16.1 Conceito

Cada atendimento pode gerar um **Ticket/Issue** que é rastreado e relacionado ao CRM.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TICKET INTEGRATION FLOW                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Cliente (WhatsApp)                                                        │
│       │                                                                     │
│       ▼                                                                     │
│  Attendance Queue ─────► Criar Ticket                                       │
│       │                  │                                                  │
│       │                  ▼                                                  │
│       │            ┌─────────────┐                                         │
│       │            │   Ticket    │                                         │
│       │            │  #TIC-001   │                                         │
│       │            │ Status: Open│                                         │
│       │            │ Priority: H │                                         │
│       │            └──────┬──────┘                                         │
│       │                   │                                                │
│       ▼                   ▼                                                │
│  Attendente           assigned_to (users table)                            │
│       │                   │                                                │
│       │                   ▼                                                │
│       │            CRM / Compliance Issues                                 │
│       │                                                                     │
│       ▼                                                                     │
│  /resolve → Ticket status = resolved                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 16.2 Modelo de Dados

#### Tabela: `attendance_tickets` (nova) ou usar `compliance_issues`

```sql
-- Opção 1: Nova tabela
CREATE TABLE attendance_tickets (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    ticket_number SERIAL,
    subject TEXT NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'open',  -- open, in_progress, pending, resolved, closed
    priority VARCHAR(20) DEFAULT 'normal',  -- low, normal, high, urgent
    category VARCHAR(50),  -- sales, support, billing, technical
    
    -- Relacionamento com users
    assigned_to UUID REFERENCES users(id),
    
    -- Relacionamento com atendente atual
    attendant_id VARCHAR(50),
    
    -- Campos de tempo
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    
    -- Integração
    channel VARCHAR(20),  -- whatsapp, telegram, web
    customer_id UUID,  -- crm_contacts
    contact_phone VARCHAR(20),
    contact_email VARCHAR(100),
    
    -- Tags e custom fields
    tags JSONB,
    custom_fields JSONB
);

-- Opção 2: Usar compliance_issues existente (recomendado)
-- Já tem: id, bot_id, title, description, status, severity, assigned_to, created_at, updated_at
```

### 16.3 Relacionamento com Users

A tabela `users` já existe:

```rust
// Schema: users table
pub struct User {
    id: Uuid,           // PK - usar em assigned_to
    username: String,
    email: String,
    password_hash: String,
    is_active: bool,
    is_admin: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

**Workflow:**
```rust
// 1. Attendant aceita atendimento
POST /api/attendance/assign
{
    "session_id": "uuid",
    "attendant_id": "att-001"
}

// 2. Sistema busca user pelo attendant
let user = users::table
    .filter(users::email.like("%attendant%"))
    .first::<User>(conn)
    .ok();

// 3. Cria/associa ticket
let ticket = AttendanceTicket {
    assigned_to: user.id,  // ← UUID da tabela users
    attendant_id: Some("att-001".to_string()),
    ..
};
```

### 16.4 Integração com CRM

O ticket pode criar/atualizar no CRM:

```basic
' Quando ticket é criado
ticket = CREATE "attendance_tickets", {
    "subject": "Problema com pedido",
    "priority": "high",
    "channel": "whatsapp",
    "customer_id": customer.id
}

' Quando resolvido
UPDATE "attendance_tickets", ticket.id, {
    "status": "resolved",
    "resolved_at": NOW()
}

' Sincroniza com CRM
CREATE "crm_deals", {
    "name": "Ticket #" + ticket.number,
    "stage": "closed_won",
    "contact_id": ticket.customer_id
}
```

### 16.5 API de Tickets

```rust
// Endpoints
GET    /api/attendance/tickets              // Listar tickets
GET    /api/attendance/tickets/{id}         // Detalhe ticket
POST   /api/attendance/tickets              // Criar ticket
PUT    /api/attendance/tickets/{id}         // Atualizar ticket
DELETE /api/attendance/tickets/{id}         // Deletar ticket

// Relacionar com atendimento
POST   /api/attendance/tickets/{id}/assign     // Atribuir a user
POST   /api/attendance/tickets/{id}/resolve    // Resolver
POST   /api/attendance/tickets/{id}/transfer   // Transferir
```

---

## 17. Integração com CRM (Pipeline de Vendas)

### 17.1 ModeloCRM Existente

O sistema já tem tables CRM:

```rust
// Estruturas existentes em contacts/crm.rs
pub struct CrmContact {
    id: Uuid,
    org_id: Uuid,
    bot_id: Uuid,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    // ... outros campos
    owner_id: Option<Uuid>,  // ← Pode usar users.id
}

pub struct CrmDeal {
    id: Uuid,
    name: String,
    value: f64,
    stage: String,  // ← Pipeline stage
    contact_id: Option<Uuid>,
    owner_id: Option<Uuid>,
}

pub struct CrmPipelineStage {
    id: Uuid,
    name: String,
    order_index: i32,
    probability: f64,
}
```

### 17.2 Integração Attendance ↔ CRM

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ATTENDANCE + CRM INTEGRATION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                │
│  │  Attendance │    │   Tickets    │    │     CRM      │                │
│  │   Queue     │    │              │    │  Pipeline    │                │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                │
│         │                    │                    │                         │
│         │                    │                    │                         │
│         ▼                    ▼                    ▼                         │
│  ┌─────────────────────────────────────────────────────────────────┐       │
│  │                     shared users table                           │       │
│  │                     (assigned_to → users.id)                    │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│                                                                             │
│  Fluxo:                                                                    │
│  1. Attendance cria Ticket                                                 │
│  2. Ticket.assigned_to = users.id                                         │
│  3. CRM Deal pode referenciar Contact do ticket                            │
│  4. Pipeline stages controlam status                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 17.3 Pipeline de Vendas no Attendance

```basic
' Configurar pipeline stages
' already exists: crm_pipeline_stages table

' Criar Deal a partir do atendimento
IF intent = "comprar" OR intent = "interesse" THEN
    ' Identifica ou cria contato
    contact = FIND "crm_contacts", "phone='" + session.phone + "'"
    
    IF contact NOT FOUND THEN
        contact = CREATE "crm_contacts", {
            "first_name": session.user_name,
            "phone": session.phone,
            "source": "whatsapp"
        }
    END IF
    
    ' Cria deal no pipeline
    deal = CREATE "crm_deals", {
        "name": "Oportunidade - " + contact.first_name,
        "contact_id": contact.id,
        "stage": "qualification",
        "owner_id": ticket.assigned_to
    }
    
    TALK "Perfeito! Vou criar uma proposta para você."
END IF
```

### 17.4 Dashboard Unificado

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ATTENDANCE + CRM DASHBOARD                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────┐  ┌──────────────────────────────────────────────┐  │
│  │   ATENDIMENTOS     │  │              PIPELINE CRM                    │  │
│  │   ─────────────    │  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │  │
│  │   Hoje: 45         │  │  │New  │ │Qual │ │Prop │ │Neg  │ │Won  │  │  │
│  │   Resolvidos: 38   │  │  │ $5K │ │$12K│ │$20K│ │$8K │ │$15K│  │  │
│  │   Em aberto: 7     │  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │  │
│  │   Tempo médio: 8min│  │                                                │  │
│  └─────────────────────┘  └──────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  TICKETS RECENTES                                                     │  │
│  │  ─────────────────────────────────────────────────────────────────── │  │
│  │  #TIC-001 | João Silva | Suporte | Alta | Maria | Aberto           │  │
│  │  #TIC-002 | Ana Costa  | Vendas  | Média| João  | Pendente         │  │
│  │  #TIC-003 | Carlos     | Técnico | Baixa| Maria | Resolvido        │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 18. Resumo: O que Faltava

| Feature | Status | Descrição |
|---------|--------|-----------|
| **Kanban View** | 🔜 Planejado | View visual da fila com drag-drop |
| **Tickets (Issues)** | 🔜 Planejado | Usar compliance_issues ou nova tabela |
| **Filas via Interface** | 🔜 Planejado | CRUD de filas + membros (users) |
| **assigned_to → users** | ✅ Já existe | users.id como FK |
| **CRM Pipeline** | ✅ Já existe | crm_deals + crm_pipeline_stages |

### 18.1 Modelo Novo (Sem attendant.csv)

```
attendance_queues
  ├── name: "Suporte WhatsApp"
  ├── channels: ["whatsapp"]
  └── members: [user_id, ...]  ← users table

attendance_queue_members
  ├── queue_id: attendance_queues.id
  ├── user_id: users.id  ← Attendente
  └── max_conversations: 5
```

### 18.2 Fluxo Completo

```
Cliente WhatsApp "Oi"
    │
    ▼
Identifica cliente no CRM (por phone)
    │
    ▼
Busca fila pelo canal → "Suporte WhatsApp"
    │
    ▼
Seleciona próximo atendente (round-robin)
    │
    ▼
Session.assigned_to = users.id
Session.customer_id = crm_contacts.id
    │
    ▼
Kanban: Card em "Novos"
    │
    ▼
Attendente aceita → Card move para "Em Atendimento"
    │
    ▼
Attendente responde
    │
    ▼
resolve → Card move para "Resolvidos"
    │
    ▼
Ticket criado com:
  - assigned_to = users.id
  - customer_id = crm_contacts.id
```

### 18.3 Próximos Passos

1. **Criar tabelas** `attendance_queues` e `attendance_queue_members`
2. **Criar UI** para gerenciar filas e membros
3. **Criar API** Kanban
4. **Adaptar Tickets** para usar users.id
5. **Dashboard Unificado** Attendance + CRM

---

## 19. Comparação com Enterprise Grade (Zendesk, Freshdesk, Intercom)

### 19.1 Matriz de Features

| Feature | Ours (Planned) | Zendesk | Freshdesk | Intercom | Priority |
|---------|---------------|---------|-----------|----------|----------|
| **CANAIS** |||||
| WhatsApp | ✅ | ✅ | ✅ | ✅ | Alta |
| Telegram | ✅ | ✅ | ✅ | ✅ | Alta |
| Instagram | ✅ | ✅ | ✅ | ✅ | Alta |
| Web Chat | ✅ | ✅ | ✅ | ✅ | Alta |
| Email | ✅ | ✅ | ✅ | ✅ | Alta |
| SMS | ✅ | ✅ | ✅ | ❌ | Média |
| Teams | ✅ | ✅ | ✅ | ❌ | Alta |
| Voice/Phone | 🔜 | ✅ | ✅ | ✅ | Alta |
| Facebook Messenger | ✅ | ✅ | ✅ | ✅ | Média |
| **TICKETING** |||||
| Criação automática | ✅ | ✅ | ✅ | ✅ | Alta |
| Status workflow | ✅ | ✅ | ✅ | ✅ | Alta |
| Prioridades | ✅ | ✅ | ✅ | ✅ | Alta |
| Categorias/Tags | ✅ | ✅ | ✅ | ✅ | Alta |
| assigned_to → users | ✅ | ✅ | ✅ | ✅ | Alta |
| Ticket relacional CRM | ✅ | ✅ | ✅ | ✅ | Alta |
| **ATENDIMENTO** |||||
| Filas (Queues) | ✅ | ✅ | ✅ | ✅ | Alta |
| Round-robin | ✅ | ✅ | ✅ | ✅ | Alta |
| Skills-based routing | 🔜 | ✅ | ✅ | ❌ | Alta |
| Kanban View | 🔜 | ✅ | ✅ | ❌ | Alta |
| Chat em tempo real | ✅ | ✅ | ✅ | ✅ | Alta |
| **AI/AUTOMAÇÃO** |||||
| Sentiment analysis | ✅ | ✅ | ✅ | ✅ | Alta |
| Smart replies | ✅ | ✅ | ✅ | ✅ | Alta |
| Auto-responder | 🔜 | ✅ | ✅ | ✅ | Alta |
| Resumo IA | ✅ | ✅ | ✅ | ✅ | Alta |
| Tips para atendente | ✅ | ✅ | ✅ | ✅ | Alta |
| **CRM** |||||
| Integração CRM | ✅ | ✅ | ✅ | ✅ | Alta |
| 360° customer view | ✅ | ✅ | ✅ | ✅ | Alta |
| Pipeline de vendas | ✅ | ✅ | ✅ | ✅ | Alta |
| Criar Deal do ticket | 🔜 | ✅ | ✅ | ✅ | Alta |
| **SLA** |||||
| SLA rules | 🔜 | ✅ | ✅ | ✅ | Alta |
| Alerts de SLA | 🔜 | ✅ | ✅ | ✅ | Alta |
| **DASHBOARD** |||||
| Métricas básicas | ✅ | ✅ | ✅ | ✅ | Alta |
| Relatórios custom | 🔜 | ✅ | ✅ | ✅ | Média |
| **KNOWLEDGE** |||||
| Base de conhecimento | 🔜 | ✅ | ✅ | ✅ | Média |
| FAQ auto | 🔜 | ✅ | ✅ | ✅ | Média |
| **VIDEO** |||||
| Videochamada | ✅ | ✅ | ✅ | ✅ | Alta |
| Screen share | ✅ | ✅ | ✅ | ✅ | Alta |
| **INTEGRAÇÕES** |||||
| Webhooks | 🔜 | ✅ | ✅ | ✅ | Alta |
| API REST | ✅ | ✅ | ✅ | ✅ | Alta |

### 19.2 Gap Analysis - O que Faltando

| Feature | Complexidade | Descrição |
|---------|--------------|------------|
| **Skills-based routing** | Alta | Route baseado em habilidade do atendente |
| **Kanban View** | Média | Drag-drop entre colunas |
| **SLA Management** | Alta | Regras, alertas, métricas |
| **Auto-responder** | Média | Respostas automáticas por IA |
| **Knowledge Base** | Alta | Artigos, FAQs, busca |
| **Relatórios custom** | Média | Queries, gráficos custom |
| **Webhooks** | Média | Notificações externas |
| **Voice/Phone (PSTN)** | Alta | Integração com telefonia |

### 19.3 Comparação Detalhada

#### Ours vs Zendesk

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ZENDESK FEATURES                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅ Já temos:                      ❌ Faltando:                            │
│  ─────────────────                ──────────────                           │
│  • Multi-channel                  • SLA Management completo                 │
│  • Ticket creation               • Knowledge base                         │
│  • User assignment               • Auto-responder IA                      │
│  • Real-time chat                • Custom reporting                        │
│  • LLM assist (tips/replies)    • Webhooks                                │
│  • Video calls                   • Marketplace apps                        │
│  • CRM integration               • Customer portals                       │
│  • Kanban (planejado)                                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Ours vs Freshdesk

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      FRESHDESK FEATURES                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅是我们 (Planned):              ❌ Faltando:                            │
│  ─────────────────                ──────────────                           │
│  • Omnichannel                   • Freddy AI (auto-responder)             │
│  • Ticket lifecycle              • Knowledge base                         │
│  • Queue management              • Custom objects                         │
│  • Round-robin                   • Approval workflows                     │
│  • Skills-based (planejado)      • Portal self-service                   │
│  • CRM integration               • SLAs                                   │
│  • Video meetings                • Advanced analytics                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 19.4 Roadmap de Implementação

```
FASE 1 (Imediato - 2 semanas)
├── ✅ Filas via Interface (users)
├── ✅ assigned_to → users.id
├── 🔜 Kanban View
└── 🔜 Tickets integrados

FASE 2 (1 mês)
├── 🔜 Skills-based routing
├── 🔜 SLA Management
└── 🔜 Auto-responder IA

FASE 3 (2 meses)
├── 🔜 Knowledge Base
├── 🔜 Custom Reporting
└── 🔜 Webhooks

FASE 4 (3 meses)
├── 🔜 Voice/PSTN
├── 🔜 Portal Self-service
└── 🔜 Advanced Integrations
```

### 19.5 Conclusão

O plano atual cobre **~70% das features enterprise-grade**:

| Categoria | Cobertura |
|-----------|-----------|
| Canais | 90% |
| Ticketing | 85% |
| Atendimento | 80% |
| AI/Automação | 75% |
| CRM | 85% |
| SLA | 30% |
| Dashboard | 60% |
| Knowledge | 20% |
| Video | 90% |
| Integrações | 50% |

**Próximas prioridades:**
1. ✅ Filas via UI + users (em desenvolvimento)
2. 🔜 Kanban View
3. 🔜 Skills-based routing
4. 🔜 SLA Management
5. 🔜 Knowledge Base

---

## 14. Arquivo de Referência

Ver também:
- [Transfer to Human](03-knowledge-ai/transfer-to-human.md)
- [LLM Assist](03-knowledge-ai/attendant-llm-assist.md)
- [Attendance Queue](06-channels/attendance-queue.md)
- [WhatsApp Setup](07-user-interface/how-to/connect-whatsapp.md)
