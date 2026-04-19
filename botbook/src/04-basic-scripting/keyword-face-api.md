# Face API Keywords

General Bots provides face detection and analysis capabilities through BASIC keywords that integrate with Azure Face API and other providers.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Face API keywords enable:

- Face detection in images and video frames
- Face verification (comparing two faces)
- Facial attribute analysis (age, emotion, glasses, etc.)
- Multi-provider support (Azure, AWS Rekognition, OpenCV)

## DETECT FACES

Detect faces in an image and return their locations and attributes.

### Syntax

```bas
faces = DETECT FACES image
faces = DETECT FACES image, options
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| image | String/File | Image path, URL, or base64 data |
| options | Object | Optional detection settings |

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| return_attributes | Boolean | false | Include facial attributes |
| return_landmarks | Boolean | false | Include facial landmarks |
| detection_model | String | "detection_01" | Detection model to use |
| recognition_model | String | "recognition_04" | Recognition model for face IDs |

### Return Value

Returns an array of detected faces:

```json
[
  {
    "face_id": "uuid",
    "rectangle": {
      "top": 100,
      "left": 150,
      "width": 200,
      "height": 250
    },
    "confidence": 0.98,
    "attributes": {
      "age": 32,
      "gender": "male",
      "emotion": "happy",
      "glasses": "none"
    }
  }
]
```

### Examples

#### Basic Detection

```bas
image = "/photos/team.jpg"
faces = DETECT FACES image

TALK "Found " + LEN(faces) + " face(s) in the image"

FOR EACH face IN faces
    TALK "Face at position: " + face.rectangle.left + ", " + face.rectangle.top
NEXT
```

#### With Attributes

```bas
options = #{return_attributes: true}
faces = DETECT FACES "/photos/portrait.jpg", options

IF LEN(faces) > 0 THEN
    face = faces[0]
    TALK "Detected a " + face.attributes.age + " year old person"
    TALK "Expression: " + face.attributes.emotion
END IF
```

#### Upload and Detect

```bas
ON FILE UPLOAD
    IF file.type STARTS WITH "image/" THEN
        faces = DETECT FACES file.path
        
        IF LEN(faces) = 0 THEN
            TALK "No faces detected in this image"
        ELSE IF LEN(faces) = 1 THEN
            TALK "Found 1 face"
        ELSE
            TALK "Found " + LEN(faces) + " faces"
        END IF
    END IF
END ON
```

## VERIFY FACE

Compare two faces to determine if they belong to the same person.

### Syntax

```bas
result = VERIFY FACE face_id1, face_id2
result = VERIFY FACE image1, image2
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| face_id1 | String | First face ID from detection |
| face_id2 | String | Second face ID from detection |
| image1 | String/File | First image (auto-detects face) |
| image2 | String/File | Second image (auto-detects face) |

### Return Value

```json
{
  "is_identical": true,
  "confidence": 0.92,
  "threshold": 0.5
}
```

### Examples

#### Verify Using Face IDs

```bas
faces1 = DETECT FACES "/photos/id-card.jpg"
faces2 = DETECT FACES "/photos/selfie.jpg"

IF LEN(faces1) > 0 AND LEN(faces2) > 0 THEN
    result = VERIFY FACE faces1[0].face_id, faces2[0].face_id
    
    IF result.is_identical THEN
        TALK "Match confirmed with " + ROUND(result.confidence * 100, 0) + "% confidence"
    ELSE
        TALK "Faces do not match"
    END IF
END IF
```

#### Verify Using Images Directly

```bas
id_photo = "/documents/passport.jpg"
live_photo = "/uploads/verification-selfie.jpg"

result = VERIFY FACE id_photo, live_photo

IF result.is_identical AND result.confidence > 0.8 THEN
    SET USER "verified", true
    TALK "Identity verified successfully!"
ELSE
    TALK "Verification failed. Please try again with a clearer photo."
END IF
```

#### KYC Verification Flow

```bas
TALK "Please upload a photo of your ID"
HEAR AS id_image

TALK "Now take a selfie for verification"
HEAR AS selfie_image

id_faces = DETECT FACES id_image
selfie_faces = DETECT FACES selfie_image

IF LEN(id_faces) = 0 THEN
    TALK "Could not detect a face in your ID photo"
ELSE IF LEN(selfie_faces) = 0 THEN
    TALK "Could not detect your face in the selfie"
ELSE
    result = VERIFY FACE id_faces[0].face_id, selfie_faces[0].face_id
    
    IF result.is_identical THEN
        IF result.confidence >= 0.9 THEN
            SET USER "kyc_status", "verified"
            TALK "✅ Identity verified with high confidence"
        ELSE IF result.confidence >= 0.7 THEN
            SET USER "kyc_status", "pending_review"
            TALK "⚠️ Verification needs manual review"
        ELSE
            SET USER "kyc_status", "failed"
            TALK "❌ Verification confidence too low"
        END IF
    ELSE
        SET USER "kyc_status", "failed"
        TALK "❌ Faces do not match"
    END IF
END IF
```

## ANALYZE FACE

Perform detailed analysis of facial attributes.

### Syntax

```bas
analysis = ANALYZE FACE image
analysis = ANALYZE FACE image, attributes
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| image | String/File | Image path, URL, or base64 data |
| attributes | Array | Specific attributes to analyze |

### Available Attributes

| Attribute | Description |
|-----------|-------------|
| age | Estimated age |
| gender | Detected gender |
| emotion | Primary emotion (happy, sad, angry, etc.) |
| glasses | Type of glasses (none, reading, sunglasses) |
| facial_hair | Beard, mustache, sideburns |
| hair | Hair color, bald, invisible |
| makeup | Eye makeup, lip makeup |
| accessories | Headwear, mask |
| occlusion | Face occlusion (forehead, eye, mouth) |
| blur | Image blur level |
| exposure | Image exposure level |
| noise | Image noise level |
| head_pose | Pitch, roll, yaw angles |
| smile | Smile intensity (0-1) |

### Return Value

```json
{
  "face_id": "uuid",
  "attributes": {
    "age": 28,
    "gender": "female",
    "emotion": {
      "primary": "happy",
      "scores": {
        "happy": 0.95,
        "neutral": 0.03,
        "surprise": 0.02
      }
    },
    "glasses": "none",
    "smile": 0.87,
    "facial_hair": {
      "beard": 0.0,
      "mustache": 0.0,
      "sideburns": 0.0
    },
    "head_pose": {
      "pitch": -5.2,
      "roll": 2.1,
      "yaw": -8.3
    },
    "quality": {
      "blur": "low",
      "exposure": "good",
      "noise": "low"
    }
  }
}
```

### Examples

#### Basic Analysis

```bas
analysis = ANALYZE FACE "/photos/headshot.jpg"

IF analysis != NULL THEN
    TALK "Estimated age: " + analysis.attributes.age
    TALK "Primary emotion: " + analysis.attributes.emotion.primary
    TALK "Smile score: " + ROUND(analysis.attributes.smile * 100, 0) + "%"
END IF
```

#### Specific Attributes

```bas
attributes = ["age", "emotion", "glasses"]
analysis = ANALYZE FACE image, attributes

TALK "Age: " + analysis.attributes.age
TALK "Emotion: " + analysis.attributes.emotion.primary
TALK "Glasses: " + analysis.attributes.glasses
```

#### Photo Quality Check

```bas
analysis = ANALYZE FACE uploaded_photo

quality = analysis.attributes.quality

IF quality.blur = "high" THEN
    TALK "Your photo is too blurry. Please upload a clearer image."
ELSE IF quality.exposure = "over" OR quality.exposure = "under" THEN
    TALK "Lighting is not optimal. Please use better lighting."
ELSE IF analysis.attributes.occlusion.forehead OR analysis.attributes.occlusion.eye THEN
    TALK "Part of your face is obscured. Please ensure your full face is visible."
ELSE
    TALK "Photo quality is acceptable!"
END IF
```

#### Emotion-Based Response

```bas
ON MESSAGE
    IF message.has_image THEN
        analysis = ANALYZE FACE message.image
        
        IF analysis != NULL THEN
            emotion = analysis.attributes.emotion.primary
            
            SWITCH emotion
                CASE "happy"
                    TALK "You look happy! 😊 How can I help you today?"
                CASE "sad"
                    TALK "I'm sorry you seem down. Is there anything I can help with?"
                CASE "angry"
                    TALK "I sense some frustration. Let me see how I can assist you."
                CASE "surprise"
                    TALK "Surprised? Let me know what's on your mind!"
                DEFAULT
                    TALK "How can I help you today?"
            END SWITCH
        END IF
    END IF
END ON
```

#### Customer Satisfaction Analysis

```bas
SET SCHEDULE "every day at 6pm"

photos = LIST "/store/customer-photos/today/"
emotions = #{happy: 0, neutral: 0, sad: 0, angry: 0}
total = 0

FOR EACH photo IN photos
    analysis = ANALYZE FACE photo.path
    
    IF analysis != NULL THEN
        primary_emotion = analysis.attributes.emotion.primary
        emotions[primary_emotion] = emotions[primary_emotion] + 1
        total = total + 1
    END IF
NEXT

IF total > 0 THEN
    satisfaction_rate = ROUND((emotions.happy / total) * 100, 1)
    
    report = "Daily Customer Mood Report\n"
    report = report + "========================\n"
    report = report + "Total analyzed: " + total + "\n"
    report = report + "Happy: " + emotions.happy + "\n"
    report = report + "Neutral: " + emotions.neutral + "\n"
    report = report + "Satisfaction rate: " + satisfaction_rate + "%"
    
    SEND MAIL TO "manager@store.com" SUBJECT "Daily Mood Report" BODY report
END IF
```

## Configuration

Add Face API credentials to your bot's `config.csv`:

### Azure Face API

```csv
key,value
face-api-provider,azure
azure-face-endpoint,https://your-resource.cognitiveservices.azure.com
azure-face-key,your-api-key
```

### AWS Rekognition

```csv
key,value
face-api-provider,aws
aws-access-key-id,your-access-key
aws-secret-access-key,your-secret-key
aws-region,us-east-1
```

### Local OpenCV (Offline)

```csv
key,value
face-api-provider,opencv
opencv-model-path,/models/face_detection
```

## Error Handling

```bas
ON ERROR RESUME NEXT

faces = DETECT FACES image

IF ERROR THEN
    error_msg = ERROR MESSAGE
    
    IF error_msg CONTAINS "rate limit" THEN
        WAIT 60
        faces = DETECT FACES image
    ELSE IF error_msg CONTAINS "invalid image" THEN
        TALK "Please provide a valid image file"
    ELSE
        TALK "Face detection is temporarily unavailable"
    END IF
END IF

CLEAR ERROR
```

## Privacy Considerations

Face detection and analysis involves biometric data. Ensure you:

- Obtain explicit consent before processing facial images
- Store face IDs temporarily (they expire after 24 hours)
- Do not store facial templates long-term without consent
- Comply with GDPR, CCPA, and other privacy regulations
- Provide clear privacy notices to users
- Allow users to request deletion of their facial data

## Best Practices

**Use appropriate thresholds.** For security applications, use higher confidence thresholds (0.8+). For general matching, 0.6+ may suffice.

**Handle multiple faces.** Always check the number of detected faces and handle edge cases.

**Check image quality.** Poor lighting, blur, or occlusion affects accuracy. Validate quality before verification.

**Implement rate limiting.** Face API services have rate limits. Cache results and avoid redundant calls.

**Provide fallbacks.** If face detection fails, offer alternative verification methods.

**Respect privacy.** Only collect and process facial data with user consent and for legitimate purposes.

## See Also

- [ON FILE UPLOAD](./keyword-on.md) - Handle uploaded images
- [SET USER](./keyword-set-user.md) - Store verification status
- [ON ERROR](./keyword-on-error.md) - Error handling
- [SEND MAIL](./keyword-send-mail.md) - Send reports