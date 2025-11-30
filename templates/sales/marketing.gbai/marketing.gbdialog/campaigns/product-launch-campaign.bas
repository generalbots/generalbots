REM ============================================================================
REM PRODUCT LAUNCH CAMPAIGN
REM Multi-channel marketing campaign for new product launches
REM ============================================================================
REM
REM This campaign orchestrates a complete product launch across:
REM - Email sequences (awareness, features, launch, follow-up)
REM - Social media posts (Instagram, Facebook, LinkedIn, Twitter)
REM - WhatsApp notifications
REM - Lead scoring and qualification
REM
REM Usage:
REM   RUN "campaigns/product-launch-campaign.bas"
REM
REM Required Parameters:
REM   product_name - Name of the product being launched
REM   launch_date - Date of the official launch (YYYY-MM-DD)
REM   landing_page_url - URL of the product landing page
REM
REM ============================================================================

PARAM product_name AS string LIKE "AI Assistant Pro"
PARAM launch_date AS string LIKE "2025-02-15"
PARAM landing_page_url AS string LIKE "https://example.com/new-product"
PARAM target_audience AS string LIKE "tech,saas,enterprise"

DESCRIPTION "Execute a multi-channel product launch campaign with email sequences, social media, and lead nurturing"

REM ============================================================================
REM CONFIGURATION
REM ============================================================================

' Campaign settings
campaign_id = "launch-" + REPLACE(product_name, " ", "-") + "-" + TODAY()
days_until_launch = DATEDIFF(TODAY(), launch_date, "day")

' Email templates
email_teaser_template = "product-launch-teaser"
email_features_template = "product-launch-features"
email_launch_template = "product-launch-day"
email_followup_template = "product-launch-followup"

' Social media hashtags
hashtags = "#" + REPLACE(product_name, " ", "") + " #NewProduct #Innovation #AI #Tech"

TALK "🚀 Starting Product Launch Campaign"
TALK "Product: " + product_name
TALK "Launch Date: " + launch_date
TALK "Days until launch: " + STR(days_until_launch)
TALK ""

REM ============================================================================
REM PHASE 1: AUDIENCE SEGMENTATION
REM ============================================================================

TALK "📊 Phase 1: Audience Segmentation"

' Get all leads from CRM
all_leads = FIND "leads.csv", "status = 'active'"

' Segment by engagement score
hot_leads = FILTER all_leads, "score >= 70"
warm_leads = FILTER all_leads, "score >= 40 AND score < 70"
cold_leads = FILTER all_leads, "score < 40"

TALK "  Hot leads (score >= 70): " + STR(COUNT(hot_leads))
TALK "  Warm leads (score 40-69): " + STR(COUNT(warm_leads))
TALK "  Cold leads (score < 40): " + STR(COUNT(cold_leads))

' Score leads with AI for better targeting
TALK "  Running AI lead scoring..."
FOR EACH lead IN hot_leads
    lead_data = #{
        email: lead.email,
        name: lead.name,
        company: lead.company,
        industry: lead.industry,
        company_size: lead.company_size,
        job_title: lead.job_title
    }

    score_result = AI SCORE LEAD lead_data

    IF score_result.qualified THEN
        ' Add to priority list
        SAVE "campaign_priority_leads.csv", lead.email, lead.name, score_result.score, score_result.grade
    END IF
NEXT

REM ============================================================================
REM PHASE 2: CONTENT PREPARATION
REM ============================================================================

TALK ""
TALK "📝 Phase 2: Content Preparation"

' Generate AI-powered content for social media
social_prompt = "Create engaging social media posts for launching " + product_name + ".
Target audience: " + target_audience + "
Tone: Professional but exciting
Include call-to-action to " + landing_page_url

social_content = LLM social_prompt

' Create email content with personalization
email_teaser_content = LLM "Write a teaser email for " + product_name + " launch. Build anticipation without revealing too much. Keep it under 200 words."

email_features_content = LLM "Write an email highlighting the top 5 features of " + product_name + ". Focus on benefits, not just features. Include bullet points."

email_launch_content = LLM "Write a launch day announcement email for " + product_name + ". Create urgency with early-bird pricing. Include clear CTA."

' Save generated content
SAVE "campaign_content.json", #{
    campaign_id: campaign_id,
    social: social_content,
    email_teaser: email_teaser_content,
    email_features: email_features_content,
    email_launch: email_launch_content,
    created_at: NOW()
}

TALK "  Content generated and saved"

REM ============================================================================
REM PHASE 3: EMAIL SEQUENCE SETUP
REM ============================================================================

TALK ""
TALK "📧 Phase 3: Email Sequence Setup"

' Schedule teaser emails (7 days before launch)
teaser_date = DATEADD(launch_date, -7, "day")
TALK "  Teaser emails scheduled for: " + teaser_date

' Schedule feature highlight emails (3 days before launch)
features_date = DATEADD(launch_date, -3, "day")
TALK "  Feature emails scheduled for: " + features_date

' Schedule launch day emails
TALK "  Launch emails scheduled for: " + launch_date

' Schedule follow-up emails (2 days after launch)
followup_date = DATEADD(launch_date, 2, "day")
TALK "  Follow-up emails scheduled for: " + followup_date

' Create scheduled jobs for email sequences
SET SCHEDULE "0 9 " + DAY(teaser_date) + " " + MONTH(teaser_date) + " *", "send-teaser-emails.bas"
SET SCHEDULE "0 9 " + DAY(features_date) + " " + MONTH(features_date) + " *", "send-features-emails.bas"
SET SCHEDULE "0 6 " + DAY(launch_date) + " " + MONTH(launch_date) + " *", "send-launch-emails.bas"
SET SCHEDULE "0 9 " + DAY(followup_date) + " " + MONTH(followup_date) + " *", "send-followup-emails.bas"

' Send to hot leads first (early access)
early_access_date = DATEADD(launch_date, -1, "day")
TALK "  Early access for hot leads: " + early_access_date

FOR EACH lead IN hot_leads
    SEND TEMPLATE email_teaser_template, "email", lead.email, #{
        name: lead.name,
        product_name: product_name,
        launch_date: launch_date,
        landing_page: landing_page_url,
        early_access: "true"
    }
NEXT

TALK "  Email sequences configured"

REM ============================================================================
REM PHASE 4: SOCIAL MEDIA CAMPAIGN
REM ============================================================================

TALK ""
TALK "📱 Phase 4: Social Media Campaign"

' Generate images for social posts
product_image = IMAGE "Professional product launch graphic for " + product_name + ", modern tech style, blue and white colors"

' Schedule social media posts

' Day -7: Teaser post
teaser_caption = "Something exciting is coming... 🔥 " + hashtags
POST TO INSTAGRAM AT DATEADD(launch_date, -7, "day") + " 10:00" product_image, teaser_caption
POST TO FACEBOOK AT DATEADD(launch_date, -7, "day") + " 10:00" product_image, teaser_caption
POST TO LINKEDIN AT DATEADD(launch_date, -7, "day") + " 09:00" product_image, "We've been working on something special. Stay tuned for a game-changing announcement. " + hashtags
TALK "  Teaser posts scheduled"

' Day -5: Problem/Solution post
problem_caption = "Tired of [common problem]? We have the solution. Mark your calendars for " + launch_date + " 📅 " + hashtags
POST TO INSTAGRAM AT DATEADD(launch_date, -5, "day") + " 10:00" product_image, problem_caption
POST TO TWITTER AT DATEADD(launch_date, -5, "day") + " 10:00" product_image, LEFT(problem_caption, 280)
TALK "  Problem/Solution posts scheduled"

' Day -3: Feature preview
feature_caption = "Sneak peek at " + product_name + "! 👀 Early access available for subscribers. Link in bio. " + hashtags
POST TO INSTAGRAM AT DATEADD(launch_date, -3, "day") + " 10:00" product_image, feature_caption
POST TO FACEBOOK AT DATEADD(launch_date, -3, "day") + " 10:00" product_image, feature_caption
TALK "  Feature preview posts scheduled"

' Day -1: Countdown
countdown_caption = "⏰ TOMORROW is the big day! " + product_name + " launches at 9 AM. Don't miss out! " + hashtags
POST TO "instagram,facebook,twitter,linkedin" AT DATEADD(launch_date, -1, "day") + " 18:00" product_image, countdown_caption
TALK "  Countdown posts scheduled"

' Launch Day: Multiple posts
launch_caption = "🚀 IT'S HERE! " + product_name + " is now LIVE! Get early-bird pricing for the next 48 hours. Link: " + landing_page_url + " " + hashtags
POST TO "instagram,facebook,twitter,linkedin" AT launch_date + " 09:00" product_image, launch_caption
TALK "  Launch day posts scheduled"

' Day +1: Social proof / testimonials
testimonial_caption = "The response has been incredible! Here's what early adopters are saying about " + product_name + " 💬 " + hashtags
POST TO "instagram,facebook,linkedin" AT DATEADD(launch_date, 1, "day") + " 10:00" product_image, testimonial_caption
TALK "  Post-launch posts scheduled"

TALK "  Social media campaign configured: 10+ posts scheduled"

REM ============================================================================
REM PHASE 5: WEBHOOK FOR LEAD CAPTURE
REM ============================================================================

TALK ""
TALK "🔗 Phase 5: Lead Capture Setup"

' Set up webhook for landing page form submissions
ON FORM SUBMIT "product-launch-" + REPLACE(product_name, " ", "-")
    ' This will be handled by the webhook handler script
    TALK "Form submission webhook registered"
END ON

' Create lead capture handler script content
lead_handler = "
REM Lead capture handler for " + product_name + " launch
REM Triggered by form submissions

' Get form data
name = form.name
email = form.email
company = form.company
phone = form.phone

' Score the lead
lead_data = #{
    email: email,
    name: name,
    company: company,
    source: 'product-launch-" + campaign_id + "'
}

score = SCORE LEAD lead_data

' Save to CRM
SAVE 'leads.csv', NOW(), name, email, company, phone, score.score, score.grade, 'product-launch'

' Send welcome email
SEND TEMPLATE 'product-launch-welcome', 'email', email, #{
    name: name,
    product_name: '" + product_name + "',
    launch_date: '" + launch_date + "'
}

' Send WhatsApp notification if phone provided
IF NOT ISEMPTY(phone) THEN
    SEND TEMPLATE 'product-launch-whatsapp', 'whatsapp', phone, #{
        name: name,
        product_name: '" + product_name + "'
    }
END IF

' Notify sales team for hot leads
IF score.score >= 80 THEN
    SEND_MAIL 'sales@company.com', 'Hot Lead: ' + name, 'New hot lead from product launch campaign. Score: ' + STR(score.score) + '. Email: ' + email, ''
END IF
"

WRITE ".gbdialog/on_form_submit_product-launch.bas", lead_handler
TALK "  Lead capture handler created"

REM ============================================================================
REM PHASE 6: WHATSAPP BROADCAST
REM ============================================================================

TALK ""
TALK "💬 Phase 6: WhatsApp Broadcast Setup"

' Get opted-in contacts for WhatsApp
whatsapp_contacts = FIND "contacts.csv", "whatsapp_opted_in = true"
TALK "  WhatsApp subscribers: " + STR(COUNT(whatsapp_contacts))

' Schedule WhatsApp broadcast for launch day
' Note: Requires WhatsApp Business API integration
whatsapp_message = "🚀 " + product_name + " is NOW LIVE!

We're excited to announce our newest innovation. As a valued subscriber, you get exclusive early access.

👉 " + landing_page_url + "

Limited time offer: Use code LAUNCH20 for 20% off!

Reply STOP to unsubscribe."

' Create WhatsApp broadcast job
SET SCHEDULE "0 9 " + DAY(launch_date) + " " + MONTH(launch_date) + " *", "whatsapp-broadcast.bas"

TALK "  WhatsApp broadcast scheduled"

REM ============================================================================
REM PHASE 7: ANALYTICS & TRACKING
REM ============================================================================

TALK ""
TALK "📊 Phase 7: Analytics Setup"

' Create campaign tracking record
campaign_record = #{
    campaign_id: campaign_id,
    product_name: product_name,
    launch_date: launch_date,
    created_at: NOW(),
    status: "active",
    channels: "email,instagram,facebook,linkedin,twitter,whatsapp",
    target_leads: COUNT(all_leads),
    hot_leads: COUNT(hot_leads),
    warm_leads: COUNT(warm_leads),
    scheduled_emails: 4,
    scheduled_social_posts: 10
}

SAVE "campaigns.csv", campaign_record.campaign_id, campaign_record.product_name, campaign_record.launch_date, campaign_record.status, campaign_record.target_leads

' Set up daily metrics collection
SET SCHEDULE "0 23 * * *", "collect-campaign-metrics.bas"

TALK "  Campaign tracking configured"

REM ============================================================================
REM SUMMARY
REM ============================================================================

TALK ""
TALK "============================================"
TALK "✅ PRODUCT LAUNCH CAMPAIGN CONFIGURED"
TALK "============================================"
TALK ""
TALK "Campaign ID: " + campaign_id
TALK "Product: " + product_name
TALK "Launch Date: " + launch_date
TALK ""
TALK "📧 Email Sequences:"
TALK "   - Teaser: " + teaser_date
TALK "   - Features: " + features_date
TALK "   - Launch: " + launch_date
TALK "   - Follow-up: " + followup_date
TALK ""
TALK "📱 Social Media:"
TALK "   - 10+ posts scheduled across 4 platforms"
TALK "   - Platforms: Instagram, Facebook, LinkedIn, Twitter"
TALK ""
TALK "👥 Audience:"
TALK "   - Total leads: " + STR(COUNT(all_leads))
TALK "   - Hot leads (priority): " + STR(COUNT(hot_leads))
TALK "   - WhatsApp subscribers: " + STR(COUNT(whatsapp_contacts))
TALK ""
TALK "🔗 Lead Capture:"
TALK "   - Form webhook configured"
TALK "   - Auto lead scoring enabled"
TALK "   - Sales notifications for hot leads"
TALK ""
TALK "============================================"

RETURN campaign_id
