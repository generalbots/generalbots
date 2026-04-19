REM General Bots: POST TO INSTAGRAM Keyword for Marketing Template
REM Posts images and captions to Instagram (requires Instagram Graph API)

PARAM username AS string LIKE "mycompany"
PARAM password AS string LIKE "password123"
PARAM image AS string LIKE "path/to/image.jpg"
PARAM caption AS string LIKE "Check out our new product! #marketing #business"

DESCRIPTION "Post an image with caption to Instagram account"

REM Note: Instagram requires OAuth and Facebook Business Account
REM This is a simplified implementation that shows the structure

TALK "📱 Posting to Instagram..."
TALK "Account: @" + username
TALK "Caption: " + caption

REM In production, you would:
REM 1. Get Instagram Graph API access token
REM 2. Upload image to Instagram
REM 3. Create media container
REM 4. Publish the post

REM For now, we simulate the post and save locally
post_data = NEW OBJECT
post_data.username = username
post_data.image = image
post_data.caption = caption
post_data.timestamp = NOW()
post_data.status = "pending"

REM Save to tracking file
SAVE "instagram_posts.csv", post_data.timestamp, post_data.username, post_data.caption, post_data.image, post_data.status

TALK "✅ Post prepared and saved!"
TALK "📊 Post details saved to instagram_posts.csv"
TALK ""
TALK "⚠️ Note: To actually post to Instagram, you need:"
TALK "1. Facebook Business Account"
TALK "2. Instagram Business Account"
TALK "3. Instagram Graph API Access Token"
TALK ""
TALK "Setup guide: https://developers.facebook.com/docs/instagram-api"

REM Return post data
RETURN post_data
