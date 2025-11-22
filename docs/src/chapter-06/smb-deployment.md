# 🏢 SMB Deployment Guide - Pragmatic BotServer Implementation

## Overview

This guide provides a **practical, cost-effective deployment** of BotServer for Small and Medium Businesses (SMBs), focusing on real-world use cases and pragmatic solutions without enterprise complexity.

## 📊 SMB Profile

**Target Company**: 50-500 employees
**Budget**: $500-5000/month for infrastructure
**IT Team**: 1-5 people
**Primary Needs**: Customer support, internal automation, knowledge management

## 🎯 Quick Start for SMBs

### 1. Single Server Deployment

```bash
# Simple all-in-one deployment for SMBs
# Runs on a single $40/month VPS (4 CPU, 8GB RAM)

# Clone and setup
git clone https://github.com/GeneralBots/BotServer
cd BotServer

# Configure for SMB (minimal features)
cat > .env << EOF
# Core Configuration
BOTSERVER_MODE=production
BOTSERVER_PORT=3000
DATABASE_URL=postgres://botserver:password@localhost/botserver

# Simple Authentication (no Zitadel complexity)
JWT_SECRET=$(openssl rand -hex 32)
ADMIN_EMAIL=admin@company.com
ADMIN_PASSWORD=ChangeMeNow123!

# OpenAI for simplicity (no self-hosted LLMs)
OPENAI_API_KEY=sk-...
OPENAI_MODEL=gpt-3.5-turbo  # Cost-effective

# Basic Storage (local, no S3 needed initially)
STORAGE_TYPE=local
STORAGE_PATH=/var/botserver/storage

# Email Integration (existing company email)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=bot@company.com
SMTP_PASSWORD=app-specific-password
EOF

# Build and run
cargo build --release --no-default-features --features email
./target/release/botserver
```

### 2. Docker Deployment (Recommended)

```yaml
# docker-compose.yml for SMB deployment
version: '3.8'

services:
  botserver:
    image: pragmatismo/botserver:latest
    ports:
      - "80:3000"
      - "443:3000"
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/botserver
      - REDIS_URL=redis://redis:6379
    volumes:
      - ./data:/var/botserver/data
      - ./certs:/var/botserver/certs
    depends_on:
      - db
      - redis
    restart: always

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: botserver
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: always

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: always

  # Optional: Simple backup solution
  backup:
    image: postgres:15-alpine
    volumes:
      - ./backups:/backups
    command: |
      sh -c 'while true; do
        PGPASSWORD=password pg_dump -h db -U postgres botserver > /backups/backup_$$(date +%Y%m%d_%H%M%S).sql
        find /backups -name "*.sql" -mtime +7 -delete
        sleep 86400
      done'
    depends_on:
      - db

volumes:
  postgres_data:
  redis_data:
```

## 💼 Common SMB Use Cases

### 1. Customer Support Bot

```typescript
// work/support/support.gbdialog
START_DIALOG support_flow

// Greeting and triage
HEAR customer_message
SET category = CLASSIFY(customer_message, ["billing", "technical", "general"])

IF category == "billing"
  USE_KB "billing_faqs"
  TALK "I'll help you with your billing question."
  
  // Check if answer exists in KB
  SET answer = FIND_IN_KB(customer_message)
  IF answer
    TALK answer
    TALK "Did this answer your question?"
    HEAR confirmation
    IF confirmation contains "no"
      CREATE_TASK "Review billing question: ${customer_message}"
      TALK "I've created a ticket for our billing team. Ticket #${task_id}"
    END
  ELSE
    SEND_MAIL to: "billing@company.com", subject: "Customer inquiry", body: customer_message
    TALK "I've forwarded your question to our billing team."
  END

ELSE IF category == "technical"
  USE_TOOL "ticket_system"
  SET ticket = CREATE_TICKET(
    title: customer_message,
    priority: "medium",
    category: "technical_support"
  )
  TALK "I've created ticket #${ticket.id}. Our team will respond within 4 hours."

ELSE
  USE_KB "general_faqs"
  TALK "Let me find that information for you..."
  // Continue with general flow
END

END_DIALOG
```

### 2. HR Assistant Bot

```typescript
// work/hr/hr.gbdialog
START_DIALOG hr_assistant

// Employee self-service
HEAR request
SET topic = EXTRACT_TOPIC(request)

SWITCH topic
  CASE "time_off":
    USE_KB "pto_policy"
    TALK "Here's our PTO policy information..."
    USE_TOOL "calendar_check"
    SET available_days = CHECK_PTO_BALANCE(user.email)
    TALK "You have ${available_days} days available."
    
    TALK "Would you like to submit a time-off request?"
    HEAR response
    IF response contains "yes"
      TALK "Please provide the dates:"
      HEAR dates
      CREATE_TASK "PTO Request from ${user.name}: ${dates}"
      SEND_MAIL to: "hr@company.com", subject: "PTO Request", body: "..."
      TALK "Your request has been submitted for approval."
    END
    
  CASE "benefits":
    USE_KB "benefits_guide"
    TALK "I can help you with benefits information..."
    
  CASE "payroll":
    TALK "For payroll inquiries, please contact HR directly at hr@company.com"
    
  DEFAULT:
    TALK "I can help with time-off, benefits, and general HR questions."
END

END_DIALOG
```

### 3. Sales Assistant Bot

```typescript
// work/sales/sales.gbdialog
START_DIALOG sales_assistant

// Lead qualification
SET lead_data = {}

TALK "Thanks for your interest! May I have your name?"
HEAR name
SET lead_data.name = name

TALK "What's your company name?"
HEAR company
SET lead_data.company = company

TALK "What's your primary need?"
HEAR need
SET lead_data.need = need

TALK "What's your budget range?"
HEAR budget
SET lead_data.budget = budget

// Score the lead
SET score = CALCULATE_LEAD_SCORE(lead_data)

IF score > 80
  // Hot lead - immediate notification
  SEND_MAIL to: "sales@company.com", priority: "high", subject: "HOT LEAD: ${company}"
  USE_TOOL "calendar_booking"
  TALK "Based on your needs, I'd like to schedule a call with our sales team."
  SET slots = GET_AVAILABLE_SLOTS("sales_team", next_2_days)
  TALK "Available times: ${slots}"
  HEAR selection
  BOOK_MEETING(selection, lead_data)
  
ELSE IF score > 50
  // Warm lead - nurture
  USE_KB "product_info"
  TALK "Let me share some relevant information about our solutions..."
  ADD_TO_CRM(lead_data, status: "nurturing")
  
ELSE
  // Cold lead - basic info
  TALK "Thanks for your interest. I'll send you our product overview."
  SEND_MAIL to: lead_data.email, template: "product_overview"
END

END_DIALOG
```

## 🔧 SMB Configuration Examples

### Simple Authentication (No Zitadel)

```rust
// src/auth/simple_auth.rs - Pragmatic auth for SMBs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{encode, decode, Header, Validation};

pub struct SimpleAuth {
    users: HashMap<String, User>,
    jwt_secret: String,
}

impl SimpleAuth {
    pub async fn login(&self, email: &str, password: &str) -> Result<Token> {
        // Simple email/password authentication
        let user = self.users.get(email).ok_or("User not found")?;
        
        // Verify password with Argon2
        let parsed_hash = PasswordHash::new(&user.password_hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
        
        // Generate simple JWT
        let claims = Claims {
            sub: email.to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            role: user.role.clone(),
        };
        
        let token = encode(&Header::default(), &claims, &self.jwt_secret)?;
        Ok(Token { access_token: token })
    }
    
    pub async fn create_user(&mut self, email: &str, password: &str, role: &str) -> Result<()> {
        // Simple user creation for SMBs
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        
        self.users.insert(email.to_string(), User {
            email: email.to_string(),
            password_hash: hash,
            role: role.to_string(),
            created_at: Utc::now(),
        });
        
        Ok(())
    }
}
```

### Local File Storage (No S3)

```rust
// src/storage/local_storage.rs - Simple file storage for SMBs
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub async fn store(&self, key: &str, data: &[u8]) -> Result<String> {
        let path = self.base_path.join(key);
        
        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Write file
        fs::write(&path, data).await?;
        
        // Return local URL
        Ok(format!("/files/{}", key))
    }
    
    pub async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.base_path.join(key);
        Ok(fs::read(path).await?)
    }
}
```

## 📊 Cost Breakdown for SMBs

### Monthly Costs (USD)

| Component | Basic | Standard | Premium |
|-----------|-------|----------|---------|
| **VPS/Cloud** | $20 | $40 | $100 |
| **Database** | Included | $20 | $50 |
| **OpenAI API** | $50 | $200 | $500 |
| **Email Service** | Free* | $10 | $30 |
| **Backup Storage** | $5 | $10 | $20 |
| **SSL Certificate** | Free** | Free** | $20 |
| **Domain** | $1 | $1 | $5 |
| **Total** | **$76** | **$281** | **$725** |

*Using company Gmail/Outlook
**Using Let's Encrypt

### Recommended Tiers

- **Basic** (< 50 employees): Single bot, 1000 conversations/month
- **Standard** (50-200 employees): Multiple bots, 10k conversations/month
- **Premium** (200-500 employees): Unlimited bots, 50k conversations/month

## 🚀 Migration Path

### Phase 1: Basic Bot (Month 1)
```bash
# Start with single customer support bot
- Deploy on $20/month VPS
- Use SQLite initially
- Basic email integration
- Manual KB updates
```

### Phase 2: Add Features (Month 2-3)
```bash
# Expand capabilities
- Migrate to PostgreSQL
- Add Redis for caching
- Implement ticket system
- Add more KB folders
```

### Phase 3: Scale (Month 4-6)
```bash
# Prepare for growth
- Move to $40/month VPS
- Add backup system
- Implement monitoring
- Add HR/Sales bots
```

### Phase 4: Optimize (Month 6+)
```bash
# Improve efficiency
- Add vector search
- Implement caching
- Optimize prompts
- Add analytics
```

## 🛠️ Maintenance Checklist

### Daily
- [ ] Check bot availability
- [ ] Review error logs
- [ ] Monitor API usage

### Weekly
- [ ] Update knowledge bases
- [ ] Review conversation logs
- [ ] Check disk space
- [ ] Test backup restoration

### Monthly
- [ ] Update dependencies
- [ ] Review costs
- [ ] Analyze bot performance
- [ ] User satisfaction survey

## 📈 KPIs for SMBs

### Customer Support
- **Response Time**: < 5 seconds
- **Resolution Rate**: > 70%
- **Escalation Rate**: < 30%
- **Customer Satisfaction**: > 4/5

### Cost Savings
- **Tickets Automated**: > 60%
- **Time Saved**: 20 hours/week
- **Cost per Conversation**: < $0.10
- **ROI**: > 300%

## 🔍 Monitoring Setup

### Simple Monitoring Stack

```yaml
# monitoring/docker-compose.yml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=redis-datasource
```

### Health Check Endpoint

```rust
// src/api/health.rs
pub async fn health_check() -> impl IntoResponse {
    let status = json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": get_uptime(),
        "memory_usage": get_memory_usage(),
        "active_sessions": get_active_sessions(),
        "database": check_database_connection(),
        "redis": check_redis_connection(),
    });
    
    Json(status)
}
```

## 📞 Support Resources

### Community Support
- Discord: https://discord.gg/generalbots
- Forum: https://forum.generalbots.com
- Docs: https://docs.generalbots.com

### Professional Support
- Email: support@pragmatismo.com.br
- Phone: +55 11 1234-5678
- Response Time: 24 hours (business days)

### Training Options
- Online Course: $99 (self-paced)
- Workshop: $499 (2 days, virtual)
- Onsite Training: $2999 (3 days)

## 🎓 Next Steps

1. **Start Small**: Deploy basic customer support bot
2. **Learn by Doing**: Experiment with dialogs and KBs
3. **Iterate Quickly**: Update based on user feedback
4. **Scale Gradually**: Add features as needed
5. **Join Community**: Share experiences and get help

## 📝 License Considerations

- **AGPL-3.0**: Open source, must share modifications
- **Commercial License**: Available for proprietary use
- **SMB Discount**: 50% off for companies < 100 employees

Contact sales@pragmatismo.com.br for commercial licensing.