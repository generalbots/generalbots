# General Bots Testing Strategy

**Version:** 6.1.0  
**Purpose:** Comprehensive testing strategy for the General Bots platform

---

## Table of Contents

1. [Overview](#overview)
2. [Test Architecture](#test-architecture)
3. [Test Categories](#test-categories)
4. [Test Accounts Setup](#test-accounts-setup)
5. [Email Testing](#email-testing)
6. [Calendar & Meeting Testing](#calendar--meeting-testing)
7. [Drive Testing](#drive-testing)
8. [Bot Response Testing](#bot-response-testing)
9. [Integration Testing](#integration-testing)
10. [Load & Performance Testing](#load--performance-testing)
11. [CI/CD Pipeline](#cicd-pipeline)
12. [Test Data Management](#test-data-management)

---

## Overview

### Testing Philosophy

Given the platform's scale and complexity (Chat, Mail, Drive, Meet, Tasks, Calendar, Analytics), we adopt a **layered testing approach**:

```
┌─────────────────────────────────────────────────────────────┐
│                    E2E Tests (10%)                          │
│            Full user journeys across apps                   │
├─────────────────────────────────────────────────────────────┤
│                Integration Tests (30%)                      │
│         Cross-service communication, APIs                   │
├─────────────────────────────────────────────────────────────┤
│                  Unit Tests (60%)                           │
│          Individual functions, modules                      │
└─────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Isolated Test Environments** - Each test run gets fresh state
2. **Real Service Testing** - Test against actual Stalwart, PostgreSQL, MinIO instances
3. **Deterministic Results** - Tests must be reproducible
4. **Fast Feedback** - Unit tests < 100ms, Integration < 5s, E2E < 30s
5. **Test Data Cleanup** - Always clean up after tests

---

## Test Architecture

### Directory Structure

```
botserver/
├── tests/
│   ├── unit/
│   │   ├── basic/          # BASIC interpreter tests
│   │   ├── email/          # Email parsing, formatting
│   │   ├── drive/          # File operations
│   │   └── llm/            # LLM integration tests
│   ├── integration/
│   │   ├── email/          # Email send/receive
│   │   ├── calendar/       # Event CRUD, invites
│   │   ├── meet/           # Video meeting lifecycle
│   │   ├── drive/          # File sharing, sync
│   │   └── bot/            # Bot responses
│   ├── e2e/
│   │   ├── scenarios/      # Full user journeys
│   │   └── smoke/          # Quick sanity checks
│   ├── fixtures/
│   │   ├── emails/         # Sample email files
│   │   ├── documents/      # Test documents
│   │   └── responses/      # Expected LLM responses
│   ├── helpers/
│   │   ├── test_accounts.rs
│   │   ├── email_client.rs
│   │   ├── calendar_client.rs
│   │   └── assertions.rs
│   └── common/
│       └── mod.rs          # Shared test utilities
```

### Test Configuration

```toml
# tests/test_config.toml
[test_environment]
database_url = "postgresql://test:test@localhost:5433/gb_test"
stalwart_url = "http://localhost:8080"
minio_endpoint = "http://localhost:9001"
livekit_url = "ws://localhost:7880"

[test_accounts]
sender_email = "sender@test.gb.local"
receiver_email = "receiver@test.gb.local"
bot_email = "bot@test.gb.local"
admin_email = "admin@test.gb.local"

[timeouts]
email_delivery_ms = 5000
meeting_join_ms = 10000
bot_response_ms = 30000
```

---

## Test Categories

### 1. Unit Tests

Fast, isolated tests for individual functions.

```rust
// tests/unit/email/signature_test.rs
#[cfg(test)]
mod tests {
    use botserver::email::signature::*;

    #[test]
    fn test_append_global_signature() {
        let body = "Hello, World!";
        let signature = "<p>-- <br>General Bots Team</p>";
        
        let result = append_signature(body, signature, SignaturePosition::Bottom);
        
        assert!(result.contains("Hello, World!"));
        assert!(result.contains("General Bots Team"));
        assert!(result.find("Hello").unwrap() < result.find("General Bots").unwrap());
    }

    #[test]
    fn test_signature_with_user_override() {
        let global_sig = "Global Signature";
        let user_sig = "User Signature";
        
        let result = combine_signatures(global_sig, Some(user_sig));
        
        // User signature should appear, global should be appended
        assert!(result.contains("User Signature"));
        assert!(result.contains("Global Signature"));
    }

    #[test]
    fn test_plain_text_signature_conversion() {
        let html_sig = "<p><b>John Doe</b><br>CEO</p>";
        let plain = html_to_plain_signature(html_sig);
        
        assert_eq!(plain, "John Doe\nCEO");
    }
}
```

### 2. Integration Tests

Test communication between services.

```rust
// tests/integration/email/send_receive_test.rs
#[tokio::test]
async fn test_email_send_and_receive() {
    let ctx = TestContext::new().await;
    
    // Create test accounts
    let sender = ctx.create_test_account("sender").await;
    let receiver = ctx.create_test_account("receiver").await;
    
    // Send email
    let email = EmailBuilder::new()
        .from(&sender.email)
        .to(&receiver.email)
        .subject("Integration Test Email")
        .body_html("<p>Test content</p>")
        .build();
    
    ctx.email_service.send(email).await.unwrap();
    
    // Wait for delivery (max 5 seconds)
    let received = ctx.wait_for_email(&receiver.email, |e| {
        e.subject == "Integration Test Email"
    }, Duration::from_secs(5)).await;
    
    assert!(received.is_some());
    assert!(received.unwrap().body.contains("Test content"));
    
    ctx.cleanup().await;
}
```

### 3. End-to-End Tests

Full user journeys across multiple apps.

```rust
// tests/e2e/scenarios/meeting_workflow_test.rs
#[tokio::test]
async fn test_complete_meeting_workflow() {
    let ctx = E2EContext::new().await;
    
    // 1. User A creates a meeting
    let host = ctx.login_as("host@test.gb.local").await;
    let meeting = host.create_meeting(MeetingConfig {
        title: "Sprint Planning",
        scheduled_at: Utc::now() + Duration::hours(1),
        participants: vec!["participant@test.gb.local"],
    }).await.unwrap();
    
    // 2. Verify invitation email was sent
    let invite_email = ctx.wait_for_email(
        "participant@test.gb.local",
        |e| e.subject.contains("Sprint Planning"),
        Duration::from_secs(10)
    ).await.unwrap();
    
    assert!(invite_email.body.contains("You've been invited"));
    assert!(invite_email.body.contains(&meeting.join_url));
    
    // 3. Participant accepts invitation
    let participant = ctx.login_as("participant@test.gb.local").await;
    participant.accept_meeting_invite(&meeting.id).await.unwrap();
    
    // 4. Verify calendar event was created for both
    let host_events = host.get_calendar_events(Utc::now(), Utc::now() + Duration::days(1)).await;
    let participant_events = participant.get_calendar_events(Utc::now(), Utc::now() + Duration::days(1)).await;
    
    assert!(host_events.iter().any(|e| e.title == "Sprint Planning"));
    assert!(participant_events.iter().any(|e| e.title == "Sprint Planning"));
    
    // 5. Start the meeting
    let room = host.start_meeting(&meeting.id).await.unwrap();
    
    // 6. Participant joins
    participant.join_meeting(&meeting.id).await.unwrap();
    
    // 7. Verify both are in the room
    let participants = ctx.get_meeting_participants(&meeting.id).await;
    assert_eq!(participants.len(), 2);
    
    // 8. Host ends meeting
    host.end_meeting(&meeting.id).await.unwrap();
    
    // 9. Verify recording is available (if enabled)
    if meeting.recording_enabled {
        let recording = ctx.wait_for_recording(&meeting.id, Duration::from_secs(30)).await;
        assert!(recording.is_some());
    }
    
    ctx.cleanup().await;
}
```

---

## Test Accounts Setup

### Account Types

| Account | Email | Purpose |
|---------|-------|---------|
| Sender | sender@test.gb.local | Initiates actions |
| Receiver | receiver@test.gb.local | Receives actions |
| Bot | bot@test.gb.local | AI bot responses |
| Admin | admin@test.gb.local | Admin operations |
| External | external@example.com | External user simulation |

### Setup Script

```bash
#!/bin/bash
# scripts/setup_test_accounts.sh

# Create test accounts in Stalwart
stalwart-cli account create sender@test.gb.local --password test123
stalwart-cli account create receiver@test.gb.local --password test123
stalwart-cli account create bot@test.gb.local --password test123
stalwart-cli account create admin@test.gb.local --password test123 --admin

# Create accounts in PostgreSQL
psql $DATABASE_URL << EOF
INSERT INTO test_accounts (account_type, email, password_hash, display_name)
VALUES 
  ('sender', 'sender@test.gb.local', '\$argon2...', 'Test Sender'),
  ('receiver', 'receiver@test.gb.local', '\$argon2...', 'Test Receiver'),
  ('bot', 'bot@test.gb.local', '\$argon2...', 'Test Bot'),
  ('admin', 'admin@test.gb.local', '\$argon2...', 'Test Admin')
ON CONFLICT (email) DO NOTHING;
EOF
```

### Test Account Helper

```rust
// tests/helpers/test_accounts.rs
pub struct TestAccount {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub account_type: AccountType,
    session: Option<Session>,
}

impl TestAccount {
    pub async fn create(ctx: &TestContext, account_type: AccountType) -> Self {
        let email = format!("{}_{:x}@test.gb.local", 
            account_type.as_str(), 
            rand::random::<u32>()
        );
        
        // Create in Stalwart via API
        ctx.stalwart_client.create_account(&email, "test123").await.unwrap();
        
        // Create in database
        let id = ctx.db.insert_test_account(&email, account_type).await.unwrap();
        
        Self {
            id,
            email,
            password: "test123".into(),
            account_type,
            session: None,
        }
    }
    
    pub async fn login(&mut self, ctx: &TestContext) -> &Session {
        let session = ctx.auth_service.login(&self.email, &self.password).await.unwrap();
        self.session = Some(session);
        self.session.as_ref().unwrap()
    }
    
    pub async fn cleanup(&self, ctx: &TestContext) {
        ctx.stalwart_client.delete_account(&self.email).await.ok();
        ctx.db.delete_test_account(&self.id).await.ok();
    }
}
```

---

## Email Testing

### Test Scenarios

#### 1. Basic Send/Receive

```rust
#[tokio::test]
async fn test_email_basic_send_receive() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Send email
    let sent = ctx.email.send(EmailRequest {
        from: sender.email.clone(),
        to: vec![receiver.email.clone()],
        subject: "Test Subject".into(),
        body_html: "<p>Test Body</p>".into(),
        body_plain: "Test Body".into(),
    }).await.unwrap();
    
    // Verify sent
    assert!(sent.message_id.is_some());
    
    // Wait for receive
    let received = ctx.wait_for_email(&receiver.email, |e| {
        e.subject == "Test Subject"
    }, Duration::from_secs(5)).await.unwrap();
    
    assert_eq!(received.from, sender.email);
    assert!(received.body_html.contains("Test Body"));
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

#### 2. Global + User Signature

```rust
#[tokio::test]
async fn test_email_signatures() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Set global signature for bot
    ctx.db.set_global_signature(ctx.bot_id, GlobalSignature {
        content_html: "<p>-- Powered by General Bots</p>".into(),
        content_plain: "-- Powered by General Bots".into(),
        position: SignaturePosition::Bottom,
    }).await.unwrap();
    
    // Set user signature
    ctx.db.set_user_signature(&sender.id, UserSignature {
        content_html: "<p>Best regards,<br>John Doe</p>".into(),
        content_plain: "Best regards,\nJohn Doe".into(),
        is_default: true,
    }).await.unwrap();
    
    // Send email
    ctx.email.send(EmailRequest {
        from: sender.email.clone(),
        to: vec![receiver.email.clone()],
        subject: "Signature Test".into(),
        body_html: "<p>Hello!</p>".into(),
        body_plain: "Hello!".into(),
        apply_signatures: true,
    }).await.unwrap();
    
    // Verify signatures in received email
    let received = ctx.wait_for_email(&receiver.email, |e| {
        e.subject == "Signature Test"
    }, Duration::from_secs(5)).await.unwrap();
    
    // Order: Body -> User Signature -> Global Signature
    let body = &received.body_html;
    let body_pos = body.find("Hello!").unwrap();
    let user_sig_pos = body.find("John Doe").unwrap();
    let global_sig_pos = body.find("General Bots").unwrap();
    
    assert!(body_pos < user_sig_pos);
    assert!(user_sig_pos < global_sig_pos);
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

#### 3. Scheduled Send

```rust
#[tokio::test]
async fn test_scheduled_email() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    let scheduled_time = Utc::now() + Duration::seconds(10);
    
    // Schedule email
    let scheduled = ctx.email.schedule(EmailRequest {
        from: sender.email.clone(),
        to: vec![receiver.email.clone()],
        subject: "Scheduled Test".into(),
        body_html: "<p>Scheduled content</p>".into(),
        scheduled_at: Some(scheduled_time),
    }).await.unwrap();
    
    assert_eq!(scheduled.status, "pending");
    
    // Verify NOT delivered yet
    tokio::time::sleep(Duration::from_secs(2)).await;
    let early_check = ctx.check_inbox(&receiver.email).await;
    assert!(!early_check.iter().any(|e| e.subject == "Scheduled Test"));
    
    // Wait for scheduled time + buffer
    tokio::time::sleep(Duration::from_secs(12)).await;
    
    // Verify delivered
    let received = ctx.check_inbox(&receiver.email).await;
    assert!(received.iter().any(|e| e.subject == "Scheduled Test"));
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

#### 4. Email Tracking

```rust
#[tokio::test]
async fn test_email_tracking() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Send with tracking enabled
    let sent = ctx.email.send(EmailRequest {
        from: sender.email.clone(),
        to: vec![receiver.email.clone()],
        subject: "Tracked Email".into(),
        body_html: "<p>Track me</p>".into(),
        tracking_enabled: true,
    }).await.unwrap();
    
    let tracking_id = sent.tracking_id.unwrap();
    
    // Check initial status
    let status = ctx.email.get_tracking_status(&tracking_id).await.unwrap();
    assert!(!status.is_read);
    assert_eq!(status.read_count, 0);
    
    // Simulate email open (load tracking pixel)
    ctx.http_client.get(&format!(
        "{}/api/email/track/{}.gif",
        ctx.server_url,
        tracking_id
    )).send().await.unwrap();
    
    // Check updated status
    let status = ctx.email.get_tracking_status(&tracking_id).await.unwrap();
    assert!(status.is_read);
    assert_eq!(status.read_count, 1);
    assert!(status.read_at.is_some());
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

#### 5. Auto-Responder (Out of Office)

```rust
#[tokio::test]
async fn test_auto_responder() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Set up auto-responder for receiver
    ctx.email.set_auto_responder(&receiver.id, AutoResponder {
        subject: "Out of Office".into(),
        body_html: "<p>I'm currently away. Will respond when I return.</p>".into(),
        start_date: Utc::now() - Duration::hours(1),
        end_date: Utc::now() + Duration::days(7),
        is_active: true,
    }).await.unwrap();
    
    // Sync to Stalwart Sieve
    ctx.stalwart.sync_sieve_rules(&receiver.email).await.unwrap();
    
    // Send email to receiver
    ctx.email.send(EmailRequest {
        from: sender.email.clone(),
        to: vec![receiver.email.clone()],
        subject: "Question".into(),
        body_html: "<p>Can we meet tomorrow?</p>".into(),
    }).await.unwrap();
    
    // Wait for auto-response
    let auto_reply = ctx.wait_for_email(&sender.email, |e| {
        e.subject.contains("Out of Office")
    }, Duration::from_secs(10)).await;
    
    assert!(auto_reply.is_some());
    assert!(auto_reply.unwrap().body.contains("currently away"));
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

---

## Calendar & Meeting Testing

### Test Scenarios

#### 1. Meeting Invitation Flow

```rust
#[tokio::test]
async fn test_meeting_invitation_accept_decline() {
    let ctx = TestContext::new().await;
    let host = TestAccount::create(&ctx, AccountType::Sender).await;
    let participant1 = TestAccount::create(&ctx, AccountType::Receiver).await;
    let participant2 = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Host creates meeting
    let meeting = ctx.calendar.create_event(CalendarEvent {
        organizer: host.email.clone(),
        title: "Team Standup".into(),
        start_time: Utc::now() + Duration::hours(2),
        end_time: Utc::now() + Duration::hours(3),
        participants: vec![
            participant1.email.clone(),
            participant2.email.clone(),
        ],
        is_meeting: true,
    }).await.unwrap();
    
    // Wait for invitation emails
    let invite1 = ctx.wait_for_email(&participant1.email, |e| {
        e.subject.contains("Team Standup") && e.content_type.contains("text/calendar")
    }, Duration::from_secs(10)).await.unwrap();
    
    let invite2 = ctx.wait_for_email(&participant2.email, |e| {
        e.subject.contains("Team Standup")
    }, Duration::from_secs(10)).await.unwrap();
    
    // Participant 1 accepts
    ctx.calendar.respond_to_invite(&participant1.id, &meeting.id, Response::Accept).await.unwrap();
    
    // Participant 2 declines
    ctx.calendar.respond_to_invite(&participant2.id, &meeting.id, Response::Decline).await.unwrap();
    
    // Host receives response notifications
    let accept_notification = ctx.wait_for_email(&host.email, |e| {
        e.subject.contains("Accepted") && e.subject.contains("Team Standup")
    }, Duration::from_secs(10)).await;
    
    let decline_notification = ctx.wait_for_email(&host.email, |e| {
        e.subject.contains("Declined") && e.subject.contains("Team Standup")
    }, Duration::from_secs(10)).await;
    
    assert!(accept_notification.is_some());
    assert!(decline_notification.is_some());
    
    // Verify meeting participants
    let updated_meeting = ctx.calendar.get_event(&meeting.id).await.unwrap();
    assert_eq!(updated_meeting.participant_status(&participant1.email), Some(ParticipantStatus::Accepted));
    assert_eq!(updated_meeting.participant_status(&participant2.email), Some(ParticipantStatus::Declined));
    
    host.cleanup(&ctx).await;
    participant1.cleanup(&ctx).await;
    participant2.cleanup(&ctx).await;
}
```

#### 2. Video Meeting Lifecycle

```rust
#[tokio::test]
async fn test_video_meeting_full_lifecycle() {
    let ctx = TestContext::new().await;
    let host = TestAccount::create(&ctx, AccountType::Sender).await;
    let participant = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // 1. Create meeting room
    let room = ctx.meet.create_room(MeetingRoom {
        name: "Test Meeting Room".into(),
        host_id: host.id,
        settings: RoomSettings {
            enable_waiting_room: true,
            enable_recording: true,
            max_participants: 10,
        },
    }).await.unwrap();
    
    // 2. Host joins
    let host_token = ctx.meet.generate_token(&room.id, &host.id, TokenRole::Host).await.unwrap();
    let host_connection = ctx.livekit.connect(&room.name, &host_token).await.unwrap();
    
    assert!(host_connection.is_connected());
    
    // 3. Participant tries to join (goes to waiting room)
    let participant_token = ctx.meet.generate_token(&room.id, &participant.id, TokenRole::Participant).await.unwrap();
    
    let waiting_entry = ctx.meet.request_join(&room.id, &participant.id).await.unwrap();
    assert_eq!(waiting_entry.status, WaitingStatus::Waiting);
    
    // 4. Host admits participant
    ctx.meet.admit_participant(&room.id, &participant.id, &host.id).await.unwrap();
    
    // 5. Participant joins
    let participant_connection = ctx.livekit.connect(&room.name, &participant_token).await.unwrap();
    assert!(participant_connection.is_connected());
    
    // 6. Verify both in room
    let participants = ctx.meet.get_participants(&room.id).await.unwrap();
    assert_eq!(participants.len(), 2);
    
    // 7. Start recording
    ctx.meet.start_recording(&room.id, &host.id).await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 8. End meeting
    ctx.meet.end_meeting(&room.id, &host.id).await.unwrap();
    
    // 9. Verify recording exists
    let recording = ctx.wait_for_condition(|| async {
        ctx.meet.get_recording(&room.id).await.ok()
    }, Duration::from_secs(30)).await.unwrap();
    
    assert!(recording.file_size > 0);
    assert!(recording.duration_seconds.unwrap() >= 5);
    
    host.cleanup(&ctx).await;
    participant.cleanup(&ctx).await;
}
```

#### 3. Meeting Breakout Rooms

```rust
#[tokio::test]
async fn test_breakout_rooms() {
    let ctx = TestContext::new().await;
    let host = TestAccount::create(&ctx, AccountType::Sender).await;
    let participants: Vec<_> = (0..6).map(|_| {
        TestAccount::create(&ctx, AccountType::Receiver)
    }).collect::<FuturesUnordered<_>>().collect().await;
    
    // Create main meeting
    let meeting = ctx.meet.create_room(MeetingRoom {
        name: "Workshop".into(),
        host_id: host.id,
        settings: Default::default(),
    }).await.unwrap();
    
    // Everyone joins
    for p in &participants {
        ctx.meet.join(&meeting.id, &p.id).await.unwrap();
    }
    
    // Create breakout rooms
    let breakout1 = ctx.meet.create_breakout_room(&meeting.id, "Group A").await.unwrap();
    let breakout2 = ctx.meet.create_breakout_room(&meeting.id, "Group B").await.unwrap();
    
    // Assign participants (3 each)
    for (i, p) in participants.iter().enumerate() {
        let room = if i < 3 { &breakout1.id } else { &breakout2.id };
        ctx.meet.assign_to_breakout(room, &p.id).await.unwrap();
    }
    
    // Start breakout sessions
    ctx.meet.start_breakout_rooms(&meeting.id).await.unwrap();
    
    // Verify participants are in correct rooms
    let room1_participants = ctx.meet.get_breakout_participants(&breakout1.id).await.unwrap();
    let room2_participants = ctx.meet.get_breakout_participants(&breakout2.id).await.unwrap();
    
    assert_eq!(room1_participants.len(), 3);
    assert_eq!(room2_participants.len(), 3);
    
    // Close breakout rooms
    ctx.meet.close_breakout_rooms(&meeting.id).await.unwrap();
    
    // Verify everyone back in main room
    let main_participants = ctx.meet.get_participants(&meeting.id).await.unwrap();
    assert_eq!(main_participants.len(), 7); // 6 participants + 1 host
    
    host.cleanup(&ctx).await;
    for p in participants {
        p.cleanup(&ctx).await;
    }
}
```

---

## Drive Testing

### Test Scenarios

#### 1. File Upload and Share

```rust
#[tokio::test]
async fn test_file_upload_and_share() {
    let ctx = TestContext::new().await;
    let owner = TestAccount::create(&ctx, AccountType::Sender).await;
    let collaborator = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // Upload file
    let file_content = b"Test document content";
    let uploaded = ctx.drive.upload(UploadRequest {
        user_id: owner.id,
        filename: "test.txt".into(),
        content: file_content.to_vec(),
        content_type: "text/plain".into(),
    }).await.unwrap();
    
    assert!(uploaded.file_id.is_some());
    assert_eq!(uploaded.size, file_content.len() as i64);
    
    // Share with collaborator
    let share = ctx.drive.share(ShareRequest {
        file_id: uploaded.file_id.unwrap(),
        shared_by: owner.id,
        shared_with_user: Some(collaborator.id),
        permission: Permission::Edit,
    }).await.unwrap();
    
    assert!(share.link_token.is_some());
    
    // Verify collaborator can access
    let files = ctx.drive.list_shared_with_me(&collaborator.id).await.unwrap();
    assert!(files.iter().any(|f| f.filename == "test.txt"));
    
    // Verify collaborator can edit
    let edit_result = ctx.drive.update_content(
        &uploaded.file_id.unwrap(),
        &collaborator.id,
        b"Modified content".to_vec()
    ).await;
    
    assert!(edit_result.is_ok());
    
    owner.cleanup(&ctx).await;
    collaborator.cleanup(&ctx).await;
}
```

#### 2. Version History

```rust
#[tokio::test]
async fn test_file_version_history() {
    let ctx = TestContext::new().await;
    let user = TestAccount::create(&ctx, AccountType::Sender).await;
    
    // Upload initial version
    let uploaded = ctx.drive.upload(UploadRequest {
        user_id: user.id,
        filename: "document.txt".into(),
        content: b"Version 1".to_vec(),
        content_type: "text/plain".into(),
    }).await.unwrap();
    
    let file_id = uploaded.file_id.unwrap();
    
    // Update file multiple times
    for i in 2..=5 {
        ctx.drive.update_content(
            &file_id,
            &user.id,
            format!("Version {}", i).into_bytes()
        ).await.unwrap();
    }
    
    // Get version history
    let versions = ctx.drive.get_versions(&file_id).await.unwrap();
    
    assert_eq!(versions.len(), 5);
    assert_eq!(versions[0].version_number, 1);
    assert_eq!(versions[4].version_number, 5);
    
    // Restore to version 2
    ctx.drive.restore_version(&file_id, 2, &user.id).await.unwrap();
    
    // Verify content
    let content = ctx.drive.download(&file_id).await.unwrap();
    assert_eq!(String::from_utf8(content.data).unwrap(), "Version 2");
    
    // New version should be 6
    let versions = ctx.drive.get_versions(&file_id).await.unwrap();
    assert_eq!(versions.len(), 6);
    
    user.cleanup(&ctx).await;
}
```

#### 3. Offline Sync

```rust
#[tokio::test]
async fn test_offline_sync_conflict() {
    let ctx = TestContext::new().await;
    let user = TestAccount::create(&ctx, AccountType::Sender).await;
    let device1 = "device_desktop";
    let device2 = "device_laptop";
    
    // Upload file
    let uploaded = ctx.drive.upload(UploadRequest {
        user_id: user.id,
        filename: "shared.txt".into(),
        content: b"Original".to_vec(),
        content_type: "text/plain".into(),
    }).await.unwrap();
    
    let file_id = uploaded.file_id.unwrap();
    
    // Mark as synced on both devices
    ctx.drive.mark_synced(&file_id, &user.id, device1, 1).await.unwrap();
    ctx.drive.mark_synced(&file_id, &user.id, device2, 1).await.unwrap();
    
    // Simulate offline edits on both devices
    ctx.drive.report_local_change(&file_id, &user.id, device1, b"Edit from desktop".to_vec()).await.unwrap();
    ctx.drive.report_local_change(&file_id, &user.id, device2, b"Edit from laptop".to_vec()).await.unwrap();
    
    // Sync device1 first
    let sync1 = ctx.drive.sync(&file_id, &user.id, device1).await.unwrap();
    assert_eq!(sync1.status, SyncStatus::Synced);
    
    // Sync device2 - should detect conflict
    let sync2 = ctx.drive.sync(&file_id, &user.id, device2).await.unwrap();
    assert_eq!(sync2.status, SyncStatus::Conflict);
    assert!(sync2.conflict_data.is_some());
    
    // Resolve conflict
    ctx.drive.resolve_conflict(&file_id, &user.id, device2, ConflictResolution::KeepBoth).await.unwrap();
    
    // Verify both versions exist
    let files = ctx.drive.list(&user.id, "/").await.unwrap();
    assert!(files.iter().any(|f| f.filename == "shared.txt"));
    assert!(files.iter().any(|f| f.filename.contains("conflict")));
    
    user.cleanup(&ctx).await;
}
```

---

## Bot Response Testing

### Test Scenarios

#### 1. Bot Responds to Email Content

```rust
#[tokio::test]
async fn test_bot_email_response() {
    let ctx = TestContext::new().await;
    let user = TestAccount::create(&ctx, AccountType::Sender).await;
    let bot = ctx.get_test_bot().await;
    
    // Send email to bot
    ctx.email.send(EmailRequest {
        from: user.email.clone(),
        to: vec![bot.email.clone()],
        subject: "Question about pricing".into(),
        body_html: "<p>What are your enterprise pricing options?</p>".into(),
    }).await.unwrap();
    
    // Wait for bot response
    let response = ctx.wait_for_email(&user.email, |e| {
        e.from == bot.email && e.subject.contains("Re: Question about pricing")
    }, Duration::from_secs(30)).await;
    
    assert!(response.is_some());
    let response = response.unwrap();
    
    // Verify response quality
    assert!(response.body.to_lowercase().contains("pricing") || 
            response.body.to_lowercase().contains("enterprise") ||
            response.body.to_lowercase().contains("plan"));
    
    // Verify response uses KB content
    let kb_keywords = ["contact sales", "custom quote", "enterprise tier"];
    assert!(kb_keywords.iter().any(|kw| response.body.to_lowercase().contains(kw)));
    
    user.cleanup(&ctx).await;
}
```

#### 2. Bot with KB Context

```rust
#[tokio::test]
async fn test_bot_kb_integration() {
    let ctx = TestContext::new().await;
    let user = TestAccount::create(&ctx, AccountType::Sender).await;
    let bot = ctx.get_test_bot().await;
    
    // Add document to KB
    ctx.kb.add_document(&bot.id, Document {
        title: "Product FAQ".into(),
        content: "Q: What is the return policy? A: 30-day money-back guarantee.".into(),
        collection: "faq".into(),
    }).await.unwrap();
    
    // Send question that should match KB
    ctx.email.send(EmailRequest {
        from: user.email.clone(),
        to: vec![bot.email.clone()],
        subject: "Return policy question".into(),
        body_html: "<p>Can I return my purchase?</p>".into(),
    }).await.unwrap();
    
    // Wait for response
    let response = ctx.wait_for_email(&user.email, |e| {
        e.from == bot.email
    }, Duration::from_secs(30)).await.unwrap();
    
    // Should contain KB information
    assert!(response.body.contains("30-day") || response.body.contains("money-back"));
    
    user.cleanup(&ctx).await;
}
```

#### 3. Bot Multi-turn Conversation

```rust
#[tokio::test]
async fn test_bot_conversation_context() {
    let ctx = TestContext::new().await;
    let user = TestAccount::create(&ctx, AccountType::Sender).await;
    let bot = ctx.get_test_bot().await;
    
    // First message
    ctx.chat.send(&user.id, &bot.id, "My name is John").await.unwrap();
    
    // Wait for acknowledgment
    ctx.wait_for_chat_response(&user.id, &bot.id, Duration::from_secs(10)).await.unwrap();
    
    // Second message - should remember name
    ctx.chat.send(&user.id, &bot.id, "What is my name?").await.unwrap();
    
    // Wait for response
    let response = ctx.wait_for_chat_response(&user.id, &bot.id, Duration::from_secs(10)).await.unwrap();
    
    // Should remember the name
    assert!(response.content.to_lowercase().contains("john"));
    
    user.cleanup(&ctx).await;
}
```

---

## Integration Testing

### Multi-Service Workflows

#### 1. Email → Calendar → Meet

```rust
#[tokio::test]
async fn test_email_to_meeting_workflow() {
    let ctx = TestContext::new().await;
    let organizer = TestAccount::create(&ctx, AccountType::Sender).await;
    let attendee = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // 1. Organizer sends meeting request via email with .ics
    let meeting_time = Utc::now() + Duration::hours(24);
    let ics = generate_ics_invite(IcsConfig {
        organizer: &organizer.email,
        attendee: &attendee.email,
        title: "Project Review",
        start: meeting_time,
        duration: Duration::hours(1),
    });
    
    ctx.email.send(EmailRequest {
        from: organizer.email.clone(),
        to: vec![attendee.email.clone()],
        subject: "Meeting: Project Review".into(),
        body_html: "<p>Please join our project review meeting.</p>".into(),
        attachments: vec![Attachment {
            filename: "invite.ics".into(),
            content_type: "text/calendar".into(),
            data: ics.into_bytes(),
        }],
    }).await.unwrap();
    
    // 2. Attendee receives and accepts
    let invite = ctx.wait_for_email(&attendee.email, |e| {
        e.subject.contains("Project Review")
    }, Duration::from_secs(10)).await.unwrap();
    
    // Process ICS attachment
    ctx.calendar.process_ics_invite(&attendee.id, &invite.attachments[0].data).await.unwrap();
    
    // 3. Verify calendar event created
    let events = ctx.calendar.get_events(&attendee.id, 
        Utc::now(), 
        Utc::now() + Duration::days(2)
    ).await.unwrap();
    
    assert!(events.iter().any(|e| e.title == "Project Review"));
    
    // 4. At meeting time, verify meeting room is available
    let event = events.iter().find(|e| e.title == "Project Review").unwrap();
    let meeting_room = ctx.meet.get_room_for_event(&event.id).await.unwrap();
    
    assert!(meeting_room.is_some());
    
    organizer.cleanup(&ctx).await;
    attendee.cleanup(&ctx).await;
}
```

#### 2. Chat → Drive → Email

```rust
#[tokio::test]
async fn test_chat_file_share_workflow() {
    let ctx = TestContext::new().await;
    let sender = TestAccount::create(&ctx, AccountType::Sender).await;
    let receiver = TestAccount::create(&ctx, AccountType::Receiver).await;
    
    // 1. User uploads file via chat
    let upload_message = ctx.chat.send_with_attachment(
        &sender.id,
        &receiver.id,
        "Here's the report",
        Attachment {
            filename: "report.pdf".into(),
            content_type: "application/pdf".into(),
            data: include_bytes!("../fixtures/documents/sample.pdf").to_vec(),
        }
    ).await.unwrap();
    
    // 2. File should be stored in Drive
    let drive_files = ctx.drive.list(&sender.id, "/").await.unwrap();
    assert!(drive_files.iter().any(|f| f.filename == "report.pdf"));
    
    // 3. Receiver gets notification via email
    let notification = ctx.wait_for_email(&receiver.email, |e| {
        e.subject.contains("shared a file") || e.subject.contains("report.pdf")
    }, Duration::from_secs(10)).await;
    
    assert!(notification.is_some());
    
    // 4. Receiver can access file
    let shared_files = ctx.drive.list_shared_with_me(&receiver.id).await.unwrap();
    assert!(shared_files.iter().any(|f| f.filename == "report.pdf"));
    
    sender.cleanup(&ctx).await;
    receiver.cleanup(&ctx).await;
}
```

---

## Load & Performance Testing

### Configuration

```rust
// tests/load/config.rs
pub struct LoadTestConfig {
    pub concurrent_users: usize,
    pub duration: Duration,
    pub ramp_up: Duration,
    pub target_rps: f64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            duration: Duration::from_secs(300),
            ramp_up: Duration::from_secs(60),
            target_rps: 1000.0,
        }
    }
}
```

### Scenarios

```rust
// tests/load/email_load_test.rs
#[tokio::test]
#[ignore] // Run manually: cargo test load -- --ignored
async fn test_email_sending_load() {
    let config = LoadTestConfig {
        concurrent_users: 50,
        duration: Duration::from_secs(60),
        ..Default::default()
    };
    
    let results = run_load_test(config, |ctx, user_id| async move {
        let start = Instant::now();
        
        ctx.email.send(EmailRequest {
            from: format!("user{}@test.local", user_id),
            to: vec!["receiver@test.local".into()],
            subject: format!("Load test {}", Uuid::new_v4()),
            body_html: "<p>Test</p>".into(),
        }).await?;
        
        Ok(start.elapsed())
    }).await;
    
    // Assertions
    assert!(results.success_rate > 0.99); // 99%+ success
    assert!(results.p95_latency < Duration::from_millis(500));
    assert!(results.p99_latency < Duration::from_secs(1));
    
    println!("Load Test Results:");
    println!("  Total requests: {}", results.total_requests);
    println!("  Success rate: {:.2}%", results.success_rate * 100.0);
    println!("  Avg latency: {:?}", results.avg_latency);
    println!("  P95 latency: {:?}", results.p95_latency);
    println!("  P99 latency: {:?}", results.p99_latency);
    println!("  Throughput: {:.2} req/s", results.throughput);
}
```

---

## CI/CD Pipeline

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  DATABASE_URL: postgresql://test:test@localhost:5432/gb_test

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Run unit tests
        run: cargo test --lib -- --test-threads=4
        working-directory: botserver

  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: gb_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
          
      stalwart:
        image: stalwartlabs/mail-server:latest
        ports:
          - 8080:8080
          - 25:25
          - 143:143
          
      minio:
        image: minio/minio:latest
        ports:
          - 9000:9000
        env:
          MINIO_ROOT_USER: minioadmin
          MINIO_ROOT_PASSWORD: minioadmin
          
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Setup test environment
        run: |
          ./scripts/setup_test_accounts.sh
          ./scripts/run_migrations.sh
          
      - name: Run integration tests
        run: cargo test --test '*' -- --test-threads=1
        working-directory: botserver
        env:
          TEST_STALWART_URL: http://localhost:8080
          TEST_MINIO_ENDPOINT: http://localhost:9000

  e2e-tests:
    runs-on: ubuntu-latest
    needs: [unit-tests, integration-tests]
    steps:
      - uses: actions/checkout@v4
      
      - name: Start full environment
        run: docker-compose -f docker-compose.test.yml up -d
        
      - name: Wait for services
        run: ./scripts/wait_for_services.sh
        
      - name: Run E2E tests
        run: cargo test --test e2e -- --test-threads=1
        working-directory: botserver
        
      - name: Collect logs on failure
        if: failure()
        run: docker-compose -f docker-compose.test.yml logs > test-logs.txt
        
      - name: Upload logs
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-logs
          path: test-logs.txt
```

---

## Test Data Management

### Fixtures

```
tests/fixtures/
├── emails/
│   ├── simple.eml
│   ├── with_attachments.eml
│   ├── calendar_invite.eml
│   └── html_rich.eml
├── documents/
│   ├── sample.pdf
│   ├── spreadsheet.xlsx
│   └── presentation.pptx
├── responses/
│   ├── pricing_question.json
│   ├── support_request.json
│   └── general_inquiry.json
└── calendar/
    ├── simple_event.ics
    ├── recurring_event.ics
    └── meeting_invite.ics
```

### Cleanup Strategy

```rust
// tests/helpers/cleanup.rs
pub struct TestContext {
    created_accounts: Vec<Uuid>,
    created_files: Vec<Uuid>,
    created_events: Vec<Uuid>,
    created_emails: Vec<String>,
}

impl TestContext {
    pub async fn cleanup(&self) {
        // Cleanup in reverse order of dependencies
        
        // 1. Delete emails
        for message_id in &self.created_emails {
            self.stalwart.delete_message(message_id).await.ok();
        }
        
        // 2. Delete events
        for event_id in &self.created_events {
            self.calendar.delete_event(event_id).await.ok();
        }
        
        // 3. Delete files
        for file_id in &self.created_files {
            self.drive.permanent_delete(file_id).await.ok();
        }
        
        // 4. Delete accounts
        for account_id in &self.created_accounts {
            self.db.delete_test_account(account_id).await.ok();
            self.stalwart.delete_account_by_id(account_id).await.ok();
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Ensure cleanup runs even on panic
        if !self.created_accounts.is_empty() {
            eprintln!("WARNING: Test context dropped without cleanup!");
            // Log for manual cleanup
        }
    }
}
```

### Database Seeding

```sql
-- tests/fixtures/seed.sql
-- Test bot configuration
INSERT INTO bots (id, name, description, llm_provider, llm_config, context_provider, context_config)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Test Bot',
    'Bot for automated testing',
    'openai',
    '{"model": "gpt-5", "temperature": 0.7}',
    'qdrant',
    '{"collection": "test_kb"}'
);

-- Global signature
INSERT INTO global_email_signatures (bot_id, name, content_html, content_plain, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Default',
    '<p>-- <br>Powered by General Bots<br>www.generalbots.com</p>',
    '-- \nPowered by General Bots\nwww.generalbots.com',
    true
);
```

---

## Summary

### Test Coverage Targets

| Category | Target Coverage | Priority |
|----------|----------------|----------|
| Email Send/Receive | 95% | P0 |
| Email Signatures | 90% | P1 |
| Email Scheduling | 90% | P1 |
| Calendar Events | 90% | P0 |
| Meeting Invites | 95% | P0 |
| Video Meetings | 85% | P1 |
| File Upload/Download | 95% | P0 |
| File Sharing | 90% | P1 |
| Bot Responses | 85% | P0 |
| Multi-service Flows | 80% | P1 |

### Running Tests

```bash
# All unit tests
cargo test --lib

# Integration tests (requires services)
cargo test --test integration

# E2E tests (requires full stack)
cargo test --test e2e

# Specific test
cargo test test_email_signatures

# With logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Load tests (manual)
cargo test load -- --ignored --nocapture
```

### Monitoring Test Health

- Track flaky tests in CI
- Monitor test execution time trends
- Review coverage reports weekly
- Update fixtures when APIs change