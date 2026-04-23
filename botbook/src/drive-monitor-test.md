# Drive Monitor Test - Upload via MinIO Console

## Objective
Test the complete sync flow for bot files uploaded through MinIO Console:
1. `.gbai` bucket creation
2. `.gbdialog/*.bas` → compilation to `.ast`
3. `.gbkb/*` → indexing to Qdrant
4. Bot activation in database

## Prerequisites

### Services Running
```bash
# Check all services are healthy
curl http://localhost:8080/health        # BotServer
curl http://localhost:3000/              # BotUI
curl http://localhost:6333/collections   # Qdrant
curl http://localhost:9100/minio/health/live  # MinIO
curl http://localhost:8300/debug/healthz # Zitadel
```

### MinIO Console Access
- URL: http://localhost:9101
- User: minioadmin
- Password: minioadmin (or check `.env` for credentials)

## Test Procedure

### Step 1: Create Bot Bucket

1. Open MinIO Console: http://localhost:9101
2. Login with credentials
3. Click **"Create Bucket"**
4. Name: `testbot.gbai` (must end with `.gbai`)
5. Click **"Create Bucket"**

### Step 2: Create Dialog Folder and File (.bas)

1. Open bucket `testbot.gbai`
2. Click **"Create New Path"**
3. Path: `testbot.gbdialog`
4. Click **"Create"**
5. Navigate into `testbot.gbdialog`
6. Click **"Upload File"** or use mc command:

```bash
# Using mc CLI (MinIO Client)
mc alias set local http://localhost:9100 minioadmin minioadmin

# Create start.bas
cat > /tmp/start.bas << 'EOF'
' start.bas - Bot entry point
ADD SUGGESTION "Check Status"
ADD SUGGESTION "Create Report"
ADD SUGGESTION "Help"

TALK "Welcome to TestBot! How can I help you today?"
EOF

mc cp /tmp/start.bas local/testbot.gbai/testbot.gbdialog/start.bas
```

### Step 3: Create Knowledge Base Folder (.gbkb)

```bash
# Create KB folder and documents
mkdir -p /tmp/testbot-docs

cat > /tmp/testbot-docs/manual.txt << 'EOF'
TestBot Manual v1.0

This is the test knowledge base for TestBot.
It contains documentation that should be indexed.

Features:
- Document search via Qdrant
- Context injection for LLM
- Semantic similarity queries

Usage:
USE KB "manual" in your dialog scripts.
EOF

cat > /tmp/testbot-docs/faq.txt << 'EOF'
Frequently Asked Questions

Q: What is TestBot?
A: A test bot for validating the drive monitor sync.

Q: How do I use it?
A: Just upload files to MinIO and wait for sync.

Q: What file types are supported?
A: .txt, .pdf, .md, .docx for KB
    .bas for dialog scripts
EOF

# Upload to MinIO
mc mb local/testbot.gbai/testbot.gbkb --ignore-existing
mc cp /tmp/testbot-docs/manual.txt local/testbot.gbai/testbot.gbkb/manual.txt
mc cp /tmp/testbot-docs/faq.txt local/testbot.gbai/testbot.gbkb/faq.txt
```

### Step 4: Verify Sync

#### 4.1 Check Database for Bot Creation
```bash
# Bot should be auto-created from bucket
./botserver-stack/bin/tables/bin/psql -h localhost -U botserver -d botserver -c \
  "SELECT id, name, is_active, created_at FROM bots WHERE name = 'testbot';"
```

Expected output:
```
 id | name | is_active | created_at
----+------+-----------+-------------------------
 ...| testbot | t | 2026-04-20 ...
```

#### 4.2 Check drive_files Table
```bash
# Files should be registered in drive_files
./botserver-stack/bin/tables/bin/psql -h localhost -U botserver -d botserver -c \
  "SELECT file_path, file_type, etag, indexed FROM drive_files WHERE file_path LIKE '%testbot%';"
```

Expected output:
```
 file_path | file_type | etag | indexed
-----------+-----------+------+---------
 testbot.gbdialog/start.bas | bas | abc123... | t
 testbot.gbkb/manual.txt | txt | def456... | t
 testbot.gbkb/faq.txt | txt | ghi789... | t
```

#### 4.3 Check .ast Compilation
```bash
# Check if .bas was compiled to .ast
ls -la /opt/gbo/work/testbot.gbai/testbot.gbdialog/
```

Expected output:
```
-rw-r--r-- 1 ubuntu ubuntu 1234 Apr 20 12:00 start.ast
-rw-r--r-- 1 ubuntu ubuntu 567 Apr 20 12:00 start.bas
```

#### 4.4 Check Qdrant Collections
```bash
# Check KB indexing
curl -s http://localhost:6333/collections | jq '.result.collections[] | select(.name | contains("testbot"))'
```

Expected output:
```json
{
  "name": "testbot_manual"
}
```

Or check points:
```bash
curl -s http://localhost:6333/collections/testbot_manual/points/scroll | jq '.result.points | length'
```

#### 4.5 Check BotServer Logs
```bash
# Monitor sync activity
tail -f botserver.log | grep -i -E "testbot|sync|compile|index"
```

Expected log patterns:
```
2026-04-20... info bootstrap:Auto-creating bot 'testbot' from S3 bucket 'testbot.gbai'
2026-04-20... info drive_compiler:Compiling testbot.gbdialog/start.bas
2026-04-20... info kb:Indexing KB folder: testbot.gbkb for bot testbot
2026-04-20... info qdrant:Collection created: testbot_manual
```

### Step 5: Test Bot via Web Interface

1. Open: http://localhost:3000/testbot
2. Login with test credentials
3. Send message: "Hello"
4. Expected response includes suggestions from start.bas

### Step 6: Test KB Search

1. In chat, type: "What is TestBot?"
2. Bot should use KB context and answer from manual.txt/faq.txt

## Troubleshooting

### Files Not Syncing

**Check MinIO bucket visibility:**
```bash
mc ls local/
```

**Check BotServer S3 connection:**
```bash
tail -100 botserver.log | grep -i "s3\|minio\|bucket"
```

### .bas Not Compiling

**Check DriveCompiler status:**
```bash
tail -f botserver.log | grep -i "drive_compiler"
```

**Manual compile trigger (if needed):**
```bash
curl -X POST http://localhost:8080/api/admin/drive/compile/testbot
```

### KB Not Indexing

**Check embedding server:**
```bash
curl http://localhost:8081/v1/models
```

**Manual KB index:**
```bash
curl -X POST http://localhost:8080/api/bots/testbot/kb/index \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"kb_name": "manual"}'
```

## Expected Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ 1. MinIO Upload                                              │
│    mc cp file.bas local/testbot.gbai/testbot.gbdialog/      │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. S3 Event / Polling (DriveMonitor)                        │
│    - Detects new file in bucket                             │
│    - Extracts metadata (etag, size, modified)               │
│    - Inserts into drive_files table                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. DriveCompiler (every 30s)                                │
│    - Queries drive_files WHERE file_type='bas'              │
│    - Compiles .bas → .ast                                   │
│    - Stores in /opt/gbo/work/{bot}.gbai/                    │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. KB Indexer (triggered by drive_files.indexed=false)      │
│    - Downloads .gbkb/* files from S3                        │
│    - Chunks text, generates embeddings                      │
│    - Stores in Qdrant collection {bot}_{kb_name}            │
│    - Updates drive_files.indexed = true                     │
└─────────────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Bot Ready                                                 │
│    - WebSocket connection at ws://localhost:8080/ws/testbot │
│    - start.bas executed on connect                          │
│    - KB available for USE KB "manual"                       │
└─────────────────────────────────────────────────────────────┘
```

## Test Checklist

- [ ] MinIO Console accessible at :9101
- [ ] Bucket `testbot.gbai` created
- [ ] Folder `testbot.gbdialog` created
- [ ] File `start.bas` uploaded
- [ ] Folder `testbot.gbkb` created
- [ ] Files `manual.txt`, `faq.txt` uploaded
- [ ] Bot auto-created in database
- [ ] Files appear in `drive_files` table
- [ ] `.ast` file generated in work dir
- [ ] Qdrant collection created
- [ ] Bot accessible at http://localhost:3000/testbot
- [ ] KB search returns relevant results

## Cleanup

```bash
# Remove test bot
mc rb --force local/testbot.gbai

# Remove from database
./botserver-stack/bin/tables/bin/psql -h localhost -U botserver -d botserver -c \
  "DELETE FROM bots WHERE name = 'testbot';"

# Remove Qdrant collection
curl -X DELETE http://localhost:6333/collections/testbot_manual

# Remove work files
rm -rf /opt/gbo/work/testbot.gbai
```
