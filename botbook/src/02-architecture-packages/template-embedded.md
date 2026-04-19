# Template: Embedded Devices

Ready-to-use templates for home automation, IoT projects, and embedded displays. Copy these templates to your `.gbdialog` folder and customize.

## Template Categories

| Template | Use Case | Hardware |
|----------|----------|----------|
| **thermostat.bas** | Smart thermostat | DHT22, Relay, OLED |
| **doorbell.bas** | Smart doorbell | Button, Camera, Buzzer |
| **light-control.bas** | Voice-controlled lights | Relay, Microphone |
| **security.bas** | Motion alarm | PIR, Buzzer, Camera |
| **plant-monitor.bas** | Plant watering | Soil sensor, Pump |
| **kiosk.bas** | Information display | HDMI display |

---

## thermostat.bas

Smart thermostat with scheduling and remote control.

```bas
' ============================================================
' SMART THERMOSTAT - Temperature control with AI assistant
' Hardware: DHT22 (GPIO 4), Relay (GPIO 17), OLED (I2C 0x3C)
' ============================================================

' Configuration
targetTemp = 22           ' Target temperature in Celsius
hysteresis = 0.5          ' Prevent rapid on/off cycling
heatingPin = 17
sensorPin = 4

' Initialize hardware
GPIO MODE heatingPin, "OUTPUT"
OLED INIT 0x3C, 128, 64

' Load saved settings
savedTarget = GET BOT MEMORY "thermostat_target"
IF savedTarget <> "" THEN
    targetTemp = VAL(savedTarget)
END IF

TALK "Thermostat ready. Target: " + targetTemp + "°C"

' Main control loop
mainLoop:
    ' Read current temperature
    currentTemp = READ TEMPERATURE sensorPin
    humidity = READ HUMIDITY sensorPin
    
    ' Control logic with hysteresis
    IF currentTemp < (targetTemp - hysteresis) THEN
        GPIO SET heatingPin, HIGH
        heating = "ON"
    ELSEIF currentTemp > (targetTemp + hysteresis) THEN
        GPIO SET heatingPin, LOW
        heating = "OFF"
    END IF
    
    ' Update display
    OLED CLEAR
    OLED PRINT 0, 0, "Current:", 1
    OLED PRINT 0, 12, currentTemp + " C", 2
    OLED PRINT 0, 35, "Target: " + targetTemp + " C", 1
    OLED PRINT 0, 48, "Heat: " + heating, 1
    OLED PRINT 80, 48, humidity + "%", 1
    
    ' Log to database every 5 minutes
    IF MINUTE(NOW) MOD 5 = 0 AND SECOND(NOW) < 5 THEN
        INSERT "temperature_log", {
            "timestamp": NOW,
            "temperature": currentTemp,
            "humidity": humidity,
            "heating": heating
        }
    END IF
    
    WAIT 5
GOTO mainLoop

' ============================================================
' Voice command handler (called when user speaks)
' ============================================================
SUB handleCommand(command)
    ' Use LLM to understand intent
    intent = LLM "Parse thermostat command: '" + command + "'. Return JSON: {action: set/status/schedule, temperature: number or null}"
    intent = JSON PARSE intent
    
    SELECT CASE intent.action
        CASE "set"
            targetTemp = intent.temperature
            SET BOT MEMORY "thermostat_target", targetTemp
            TALK "Temperature set to " + targetTemp + " degrees"
            
        CASE "status"
            TALK "Current temperature is " + currentTemp + " degrees. Target is " + targetTemp + ". Heating is " + heating
            
        CASE "schedule"
            scheduleTime = intent.time
            scheduleTemp = intent.temperature
            SET SCHEDULER "thermostat_schedule", scheduleTime, "setTemperature", scheduleTemp
            TALK "Scheduled temperature change to " + scheduleTemp + " degrees at " + scheduleTime
    END SELECT
END SUB
```

---

## doorbell.bas

Smart doorbell with photo capture and notifications.

```bas
' ============================================================
' SMART DOORBELL - Ring detection with photo and notification
' Hardware: Button (GPIO 27), Buzzer (GPIO 18), LED (GPIO 17)
' Optional: Camera module, OLED display
' ============================================================

' Pin configuration
buttonPin = 27
buzzerPin = 18
ledPin = 17

' Setup
GPIO MODE buttonPin, "INPUT_PULLUP"
GPIO MODE buzzerPin, "OUTPUT"
GPIO MODE ledPin, "OUTPUT"

' Notification settings
ownerEmail = GET BOT MEMORY "doorbell_email"
IF ownerEmail = "" THEN
    ownerEmail = "owner@example.com"
END IF

TALK "Doorbell system ready"

' Visual indicator that system is active
GPIO SET ledPin, LOW

mainLoop:
    ' Check button
    IF GPIO GET buttonPin = LOW THEN
        ' Debounce
        WAIT 0.05
        IF GPIO GET buttonPin = LOW THEN
            handleRing()
            ' Wait for button release
            WHILE GPIO GET buttonPin = LOW
                WAIT 0.01
            WEND
        END IF
    END IF
    
    WAIT 0.1
GOTO mainLoop

' ============================================================
SUB handleRing()
    timestamp = FORMAT(NOW, "HH:mm:ss")
    
    ' Visual and audio feedback
    GPIO SET ledPin, HIGH
    BUZZER buzzerPin, 1000, 0.3
    WAIT 0.1
    BUZZER buzzerPin, 1500, 0.3
    
    ' Try to capture photo
    ON ERROR RESUME NEXT
    photo = CAPTURE PHOTO
    photoPath = "doorbell_" + FORMAT(NOW, "YYYYMMDD_HHmmss") + ".jpg"
    IF NOT ERROR THEN
        WRITE "photos/" + photoPath, photo
    END IF
    ON ERROR GOTO 0
    
    ' Log the ring
    INSERT "doorbell_log", {
        "timestamp": NOW,
        "photo": photoPath
    }
    
    ' Send notification
    IF photo THEN
        SEND MAIL ownerEmail, "Doorbell Ring", "Someone is at the door! Time: " + timestamp, [photo]
    ELSE
        SEND MAIL ownerEmail, "Doorbell Ring", "Someone is at the door! Time: " + timestamp
    END IF
    
    ' Announce locally
    TALK "Visitor at the door"
    
    ' LED off after 2 seconds
    WAIT 2
    GPIO SET ledPin, LOW
END SUB

' ============================================================
' Configuration command
' ============================================================
SUB setEmail(email)
    SET BOT MEMORY "doorbell_email", email
    ownerEmail = email
    TALK "Notification email set to " + email
END SUB
```

---

## light-control.bas

Voice-controlled lighting with scenes and schedules.

```bas
' ============================================================
' SMART LIGHTS - Voice and schedule controlled lighting
' Hardware: 4-channel relay on GPIO 17, 18, 27, 22
' ============================================================

' Room configuration
DIM rooms(4)
rooms(0) = "living room"
rooms(1) = "bedroom"
rooms(2) = "kitchen"
rooms(3) = "bathroom"

DIM pins(4)
pins(0) = 17
pins(1) = 18
pins(2) = 27
pins(3) = 22

' Initialize all relays
FOR i = 0 TO 3
    GPIO MODE pins(i), "OUTPUT"
    GPIO SET pins(i), LOW    ' All lights off
NEXT

TALK "Light control ready. Say 'turn on' or 'turn off' followed by room name."

' ============================================================
' Main dialog loop
' ============================================================
mainLoop:
    command = HEAR
    
    IF command <> "" THEN
        handleCommand(command)
    END IF
    
    ' Check schedules
    checkSchedules()
    
    WAIT 0.5
GOTO mainLoop

' ============================================================
SUB handleCommand(command)
    ' Use LLM to parse command
    prompt = "Parse light command: '" + command + "'. "
    prompt = prompt + "Rooms: living room, bedroom, kitchen, bathroom, all. "
    prompt = prompt + "Return JSON: {action: on/off/toggle/status/scene, room: string, scene: string or null}"
    
    intent = LLM prompt
    intent = JSON PARSE intent
    
    SELECT CASE intent.action
        CASE "on"
            turnOn(intent.room)
        CASE "off"
            turnOff(intent.room)
        CASE "toggle"
            toggle(intent.room)
        CASE "status"
            reportStatus()
        CASE "scene"
            applyScene(intent.scene)
    END SELECT
END SUB

' ============================================================
SUB turnOn(room)
    IF room = "all" THEN
        FOR i = 0 TO 3
            RELAY SET pins(i), ON
        NEXT
        TALK "All lights on"
    ELSE
        FOR i = 0 TO 3
            IF rooms(i) = room THEN
                RELAY SET pins(i), ON
                TALK room + " light on"
                EXIT FOR
            END IF
        NEXT
    END IF
END SUB

SUB turnOff(room)
    IF room = "all" THEN
        FOR i = 0 TO 3
            RELAY SET pins(i), OFF
        NEXT
        TALK "All lights off"
    ELSE
        FOR i = 0 TO 3
            IF rooms(i) = room THEN
                RELAY SET pins(i), OFF
                TALK room + " light off"
                EXIT FOR
            END IF
        NEXT
    END IF
END SUB

SUB toggle(room)
    FOR i = 0 TO 3
        IF rooms(i) = room OR room = "all" THEN
            state = GPIO GET pins(i)
            RELAY SET pins(i), NOT state
        END IF
    NEXT
    TALK room + " toggled"
END SUB

SUB reportStatus()
    status = "Light status: "
    FOR i = 0 TO 3
        state = GPIO GET pins(i)
        IF state = HIGH THEN
            status = status + rooms(i) + " ON, "
        ELSE
            status = status + rooms(i) + " off, "
        END IF
    NEXT
    TALK status
END SUB

' ============================================================
' Scenes
' ============================================================
SUB applyScene(scene)
    SELECT CASE scene
        CASE "movie"
            RELAY SET pins(0), OFF     ' Living room dim
            RELAY SET pins(1), OFF
            RELAY SET pins(2), OFF
            RELAY SET pins(3), OFF
            TALK "Movie scene applied - all lights off"
            
        CASE "morning"
            RELAY SET pins(2), ON      ' Kitchen on
            RELAY SET pins(3), ON      ' Bathroom on
            RELAY SET pins(0), OFF
            RELAY SET pins(1), OFF
            TALK "Morning scene - kitchen and bathroom on"
            
        CASE "night"
            RELAY SET pins(1), ON      ' Bedroom only
            RELAY SET pins(0), OFF
            RELAY SET pins(2), OFF
            RELAY SET pins(3), OFF
            TALK "Night scene - bedroom only"
            
        CASE "all on"
            FOR i = 0 TO 3
                RELAY SET pins(i), ON
            NEXT
            TALK "All lights on"
    END SELECT
END SUB

' ============================================================
' Schedule checking
' ============================================================
SUB checkSchedules()
    hour = HOUR(NOW)
    minute = MINUTE(NOW)
    
    ' Sunset - turn on living room
    IF hour = 18 AND minute = 0 THEN
        RELAY SET pins(0), ON
    END IF
    
    ' Bedtime - night scene
    IF hour = 22 AND minute = 30 THEN
        applyScene("night")
    END IF
    
    ' Late night - all off
    IF hour = 23 AND minute = 30 THEN
        turnOff("all")
    END IF
END SUB
```

---

## security.bas

Motion-activated security system with alerts.

```bas
' ============================================================
' SECURITY SYSTEM - Motion detection with alerts
' Hardware: PIR (GPIO 7), Buzzer (GPIO 18), LED (GPIO 17)
' Optional: Camera, OLED display
' ============================================================

' Configuration
pirPin = 7
buzzerPin = 18
ledPin = 17
alertEmail = "security@example.com"

' State
armed = FALSE
alertCooldown = 0

' Initialize
GPIO MODE pirPin, "INPUT"
GPIO MODE buzzerPin, "OUTPUT"
GPIO MODE ledPin, "OUTPUT"

TALK "Security system ready. Say 'arm' to activate."

mainLoop:
    ' Check for motion when armed
    IF armed THEN
        GPIO SET ledPin, HIGH    ' Armed indicator
        
        motion = READ MOTION pirPin
        IF motion AND NOW > alertCooldown THEN
            handleAlert()
            alertCooldown = NOW + 60    ' 1 minute cooldown
        END IF
    ELSE
        GPIO SET ledPin, LOW
    END IF
    
    ' Check for commands
    command = HEAR NOWAIT
    IF command <> "" THEN
        handleCommand(command)
    END IF
    
    WAIT 0.5
GOTO mainLoop

' ============================================================
SUB handleCommand(command)
    command = LOWER(command)
    
    IF INSTR(command, "arm") > 0 THEN
        IF INSTR(command, "disarm") > 0 THEN
            disarmSystem()
        ELSE
            armSystem()
        END IF
    ELSEIF INSTR(command, "status") > 0 THEN
        IF armed THEN
            TALK "System is armed"
        ELSE
            TALK "System is disarmed"
        END IF
    ELSEIF INSTR(command, "test") > 0 THEN
        testAlarm()
    END IF
END SUB

SUB armSystem()
    TALK "Arming in 10 seconds. Leave the area."
    
    ' Countdown beeps
    FOR i = 10 TO 1 STEP -1
        BUZZER buzzerPin, 1000, 0.1
        WAIT 1
    NEXT
    
    armed = TRUE
    BUZZER buzzerPin, 2000, 0.5
    TALK "System armed"
END SUB

SUB disarmSystem()
    armed = FALSE
    GPIO SET ledPin, LOW
    TALK "System disarmed"
END SUB

SUB handleAlert()
    TALK "Motion detected! Alerting..."
    
    ' Sound alarm
    FOR i = 1 TO 5
        BUZZER buzzerPin, 2000, 0.2
        WAIT 0.1
        BUZZER buzzerPin, 1500, 0.2
        WAIT 0.1
    NEXT
    
    ' Capture photo if camera available
    ON ERROR RESUME NEXT
    photo = CAPTURE PHOTO
    ON ERROR GOTO 0
    
    ' Log event
    INSERT "security_events", {
        "timestamp": NOW,
        "type": "motion",
        "photo": photo
    }
    
    ' Send alert
    IF photo THEN
        SEND MAIL alertEmail, "SECURITY ALERT", "Motion detected at " + FORMAT(NOW, "HH:mm:ss"), [photo]
    ELSE
        SEND MAIL alertEmail, "SECURITY ALERT", "Motion detected at " + FORMAT(NOW, "HH:mm:ss")
    END IF
END SUB

SUB testAlarm()
    TALK "Testing alarm"
    BUZZER buzzerPin, 1500, 0.5
    WAIT 0.5
    BUZZER buzzerPin, 2000, 0.5
    TALK "Test complete"
END SUB
```

---

## plant-monitor.bas

Automatic plant watering system.

```bas
' ============================================================
' PLANT MONITOR - Soil moisture monitoring and auto-watering
' Hardware: Soil sensor (ADC 0), Water pump relay (GPIO 17)
' Optional: DHT22 for air temp/humidity
' ============================================================

' Configuration
soilChannel = 0          ' ADC channel for soil sensor
pumpPin = 17             ' Relay for water pump
dryThreshold = 300       ' Below this = dry soil (adjust for your sensor)
wetThreshold = 700       ' Above this = wet enough
waterDuration = 5        ' Seconds to run pump

' Initialize
GPIO MODE pumpPin, "OUTPUT"
GPIO SET pumpPin, LOW

TALK "Plant monitor ready"

mainLoop:
    ' Read soil moisture
    moisture = READ LIGHT soilChannel    ' Using ADC reading
    
    ' Read air conditions if sensor available
    ON ERROR RESUME NEXT
    airTemp = READ TEMPERATURE 4
    airHumidity = READ HUMIDITY 4
    ON ERROR GOTO 0
    
    ' Determine soil status
    IF moisture < dryThreshold THEN
        status = "DRY"
        needsWater = TRUE
    ELSEIF moisture > wetThreshold THEN
        status = "WET"
        needsWater = FALSE
    ELSE
        status = "OK"
        needsWater = FALSE
    END IF
    
    ' Auto water if dry
    IF needsWater THEN
        waterPlant()
    END IF
    
    ' Update display
    LCD CLEAR
    LCD PRINT 0, 0, "Soil: " + status
    LCD PRINT 1, 0, "M:" + moisture
    
    ' Log every 30 minutes
    IF MINUTE(NOW) MOD 30 = 0 AND SECOND(NOW) < 10 THEN
        INSERT "plant_log", {
            "timestamp": NOW,
            "moisture": moisture,
            "status": status,
            "air_temp": airTemp,
            "air_humidity": airHumidity
        }
    END IF
    
    WAIT 60    ' Check every minute
GOTO mainLoop

' ============================================================
SUB waterPlant()
    ' Only water during daytime (6am - 8pm)
    hour = HOUR(NOW)
    IF hour < 6 OR hour > 20 THEN
        RETURN
    END IF
    
    ' Check if we watered recently (within 2 hours)
    lastWater = GET BOT MEMORY "last_water_time"
    IF lastWater <> "" THEN
        IF NOW - VAL(lastWater) < 7200 THEN    ' 2 hours in seconds
            RETURN
        END IF
    END IF
    
    TALK "Watering plant"
    
    ' Run pump
    GPIO SET pumpPin, HIGH
    WAIT waterDuration
    GPIO SET pumpPin, LOW
    
    ' Record watering
    SET BOT MEMORY "last_water_time", NOW
    
    INSERT "watering_log", {
        "timestamp": NOW,
        "duration": waterDuration
    }
    
    ' Send notification
    SEND MAIL "gardener@example.com", "Plant Watered", "Your plant was automatically watered at " + FORMAT(NOW, "HH:mm")
    
    TALK "Watering complete"
END SUB

' ============================================================
' Manual commands
' ============================================================
SUB handleCommand(command)
    IF INSTR(command, "water") > 0 THEN
        TALK "Manual watering"
        GPIO SET pumpPin, HIGH
        WAIT waterDuration
        GPIO SET pumpPin, LOW
        TALK "Done"
    ELSEIF INSTR(command, "status") > 0 THEN
        TALK "Soil moisture is " + moisture + ". Status: " + status
    END IF
END SUB
```

---

## kiosk.bas

Information display kiosk for lobbies and public spaces.

```bas
' ============================================================
' INFO KIOSK - Public information display with AI assistant
' Hardware: HDMI display, touch screen or keyboard
' ============================================================

' Configuration
welcomeMessage = "Welcome! How can I help you today?"
idleTimeout = 60    ' Return to welcome screen after inactivity

' State
lastActivity = NOW

' Initialize display
' (Runs in browser kiosk mode at http://localhost:9000/embedded/)

TALK welcomeMessage

mainLoop:
    ' Check for user input
    input = HEAR NOWAIT
    
    IF input <> "" THEN
        lastActivity = NOW
        handleQuery(input)
    END IF
    
    ' Return to welcome screen after idle
    IF NOW - lastActivity > idleTimeout THEN
        showWelcome()
        lastActivity = NOW
    END IF
    
    WAIT 0.5
GOTO mainLoop

' ============================================================
SUB handleQuery(query)
    ' Log the query
    INSERT "kiosk_queries", {
        "timestamp": NOW,
        "query": query
    }
    
    ' Determine intent
    intent = LLM "Classify query: '" + query + "'. Categories: directions, hours, services, general, weather. Return JSON: {category: string, details: string}"
    intent = JSON PARSE intent
    
    SELECT CASE intent.category
        CASE "directions"
            handleDirections(intent.details)
        CASE "hours"
            handleHours(intent.details)
        CASE "services"
            handleServices(intent.details)
        CASE "weather"
            handleWeather()
        CASE ELSE
            handleGeneral(query)
    END SELECT
END SUB

SUB handleDirections(destination)
    ' Look up location in database
    location = FIND "locations", "name LIKE '%" + destination + "%'"
    
    IF location THEN
        TALK "To get to " + location.name + ": " + location.directions
    ELSE
        TALK "I'm sorry, I don't have directions to " + destination + ". Please ask at the front desk."
    END IF
END SUB

SUB handleHours(department)
    hours = FIND "hours", "department LIKE '%" + department + "%'"
    
    IF hours THEN
        TALK hours.department + " is open " + hours.schedule
    ELSE
        TALK "Standard hours are Monday to Friday, 9 AM to 5 PM."
    END IF
END SUB

SUB handleServices(service)
    info = FIND "services", "name LIKE '%" + service + "%'"
    
    IF info THEN
        TALK info.description
    ELSE
        ' Use KB for general info
        USE KB "services"
        answer = LLM "What services are available for: " + service
        TALK answer
    END IF
END SUB

SUB handleWeather()
    weather = WEATHER "local"
    TALK "Current weather: " + weather.description + ", " + weather.temperature + " degrees."
END SUB

SUB handleGeneral(query)
    ' Use KB for general questions
    USE KB "faq"
    answer = LLM query
    TALK answer
END SUB

SUB showWelcome()
    TALK welcomeMessage
END SUB
```

---

## Usage

### Installing Templates

1. Create a `.gbdialog` folder in your bot package:
   ```bash
   mkdir -p mybot.gbai/mybot.gbdialog
   ```

2. Copy the desired template:
   ```bash
   cp template-embedded.md mybot.gbai/mybot.gbdialog/thermostat.bas
   ```

3. Edit configuration at the top of the file

4. Deploy to your device:
   ```bash
   ./scripts/deploy-embedded.sh pi@mydevice --with-ui
   ```

### Customizing

Each template has a configuration section at the top:

```bas
' Configuration
targetTemp = 22           ' Target temperature in Celsius
heatingPin = 17           ' GPIO pin for relay
sensorPin = 4             ' GPIO pin for sensor
```

Modify these values to match your hardware setup.

### Combining Templates

You can combine functionality from multiple templates into a single dialog:

```bas
' Combined home automation
INCLUDE "thermostat.bas"
INCLUDE "light-control.bas"
INCLUDE "security.bas"

' Main loop handles all systems
mainLoop:
    checkThermostat()
    checkLights()
    checkSecurity()
    WAIT 1
GOTO mainLoop
