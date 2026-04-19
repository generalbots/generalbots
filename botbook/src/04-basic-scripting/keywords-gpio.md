# GPIO & IoT Keywords

Control hardware directly from BASIC dialogs. These keywords work on Raspberry Pi, Orange Pi, and other SBCs running botserver.

## Hardware ↔ Keyword Compatibility

| Hardware Component | BASIC Keywords | Interface | Typical Address/Pin |
|-------------------|----------------|-----------|---------------------|
| **Displays** |
| OLED 0.96" SSD1306 | `OLED INIT`, `OLED PRINT`, `OLED DRAW`, `OLED CLEAR` | I2C | 0x3C |
| OLED 1.3" SH1106 | `OLED INIT`, `OLED PRINT`, `OLED DRAW`, `OLED CLEAR` | I2C | 0x3C |
| LCD 16x2 HD44780 | `LCD INIT`, `LCD PRINT`, `LCD CLEAR` | I2C | 0x27 or 0x3F |
| LCD 20x4 HD44780 | `LCD INIT`, `LCD PRINT`, `LCD CLEAR` | I2C | 0x27 or 0x3F |
| TFT ILI9341 | `SPI OPEN`, `SPI TRANSFER` | SPI | CE0 |
| E-Ink Waveshare | `SPI OPEN`, `SPI TRANSFER` | SPI | CE0 |
| **Temperature Sensors** |
| DHT11 | `READ TEMPERATURE`, `READ HUMIDITY` | GPIO | Any GPIO |
| DHT22/AM2302 | `READ TEMPERATURE`, `READ HUMIDITY` | GPIO | Any GPIO |
| DS18B20 | `READ TEMPERATURE` | 1-Wire | GPIO 4 (default) |
| BME280 | `I2C READ`, `READ TEMPERATURE`, `READ HUMIDITY` | I2C | 0x76 or 0x77 |
| BMP280 | `I2C READ`, `READ TEMPERATURE` | I2C | 0x76 or 0x77 |
| **Motion/Distance Sensors** |
| HC-SR04 Ultrasonic | `READ DISTANCE` | GPIO | Trigger + Echo pins |
| PIR HC-SR501 | `READ MOTION`, `GPIO GET` | GPIO | Any GPIO |
| PIR AM312 | `READ MOTION`, `GPIO GET` | GPIO | Any GPIO |
| **Light Sensors** |
| Photoresistor (LDR) | `READ LIGHT` | ADC | MCP3008 CH0-7 |
| BH1750 | `I2C READ`, `READ LIGHT` | I2C | 0x23 or 0x5C |
| TSL2561 | `I2C READ` | I2C | 0x29, 0x39, 0x49 |
| **Actuators** |
| Relay Module | `RELAY SET`, `GPIO SET` | GPIO | Any GPIO |
| LED | `GPIO SET`, `GPIO PWM` | GPIO | Any GPIO |
| Buzzer (Active) | `BUZZER`, `GPIO SET` | GPIO | Any GPIO |
| Buzzer (Passive) | `BUZZER`, `GPIO PWM` | GPIO | PWM pin |
| Servo Motor | `GPIO SERVO` | PWM | GPIO 12, 13, 18, 19 |
| DC Motor (via L298N) | `GPIO SET`, `GPIO PWM` | GPIO | 2 GPIO + PWM |
| Stepper (via ULN2003) | `GPIO SET` | GPIO | 4 GPIO pins |
| **Input Devices** |
| Push Button | `GPIO GET` | GPIO | Any GPIO |
| Rotary Encoder | `GPIO GET` | GPIO | CLK + DT + SW |
| Keypad 4x4 | `GPIO GET` | GPIO | 8 GPIO pins |
| **ADC/DAC** |
| MCP3008 (ADC) | `SPI TRANSFER`, `READ LIGHT` | SPI | CE0 |
| ADS1115 (ADC) | `I2C READ` | I2C | 0x48 |
| MCP4725 (DAC) | `I2C WRITE` | I2C | 0x60 |
| **Expansion** |
| PCF8574 I/O Expander | `I2C READ`, `I2C WRITE` | I2C | 0x20-0x27 |
| MCP23017 I/O Expander | `I2C READ`, `I2C WRITE` | I2C | 0x20-0x27 |

### Quick Reference by Keyword

| Keyword | Works With | Notes |
|---------|-----------|-------|
| `GPIO MODE` | All GPIO devices | Set pin as INPUT/OUTPUT |
| `GPIO SET` | LEDs, Relays, Buzzers | Digital HIGH/LOW |
| `GPIO GET` | Buttons, PIR, Digital sensors | Read digital state |
| `GPIO PWM` | LEDs, Motors, Passive buzzers | PWM output (0-100%) |
| `GPIO SERVO` | Servo motors | Angle 0-180° |
| `I2C SCAN` | All I2C devices | Find connected devices |
| `I2C READ` | I2C sensors/displays | Read bytes from register |
| `I2C WRITE` | I2C devices | Write bytes to register |
| `SPI OPEN` | SPI devices | Initialize SPI bus |
| `SPI TRANSFER` | TFT displays, ADC | Send/receive data |
| `READ TEMPERATURE` | DHT11/22, DS18B20, BME280 | Auto-detects sensor |
| `READ HUMIDITY` | DHT11/22, BME280 | Requires humidity sensor |
| `READ DISTANCE` | HC-SR04 | Ultrasonic sensor |
| `READ MOTION` | PIR sensors | Returns TRUE/FALSE |
| `READ LIGHT` | Photoresistor + ADC | Analog light level |
| `LCD INIT` | HD44780 I2C displays | Initialize 16x2 or 20x4 |
| `LCD PRINT` | HD44780 displays | Print text |
| `OLED INIT` | SSD1306, SH1106 | Initialize OLED |
| `OLED PRINT` | OLED displays | Print text with size |
| `OLED DRAW` | OLED displays | Draw shapes |
| `RELAY SET` | Relay modules | ON/OFF control |
| `BUZZER` | Active/Passive buzzers | Beep or tone |

## Overview

```
┌────────────────────────────────────────────────────────────────────────┐
│                    GPIO Keyword Architecture                            │
├────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   BASIC Dialog          botserver            Hardware                  │
│   ┌──────────┐         ┌──────────┐         ┌──────────┐              │
│   │ GPIO SET │────────▶│  sysfs/  │────────▶│   Pin    │              │
│   │ GPIO GET │◀────────│  gpiod   │◀────────│  State   │              │
│   │ I2C READ │◀────────│  i2c-dev │◀────────│  Sensor  │              │
│   │ SPI WRITE│────────▶│  spidev  │────────▶│  Display │              │
│   └──────────┘         └──────────┘         └──────────┘              │
│                                                                         │
└────────────────────────────────────────────────────────────────────────┘
```

## GPIO Keywords

### GPIO MODE

Set a GPIO pin as input or output.

```bas
' Set pin 17 as output (for LED, relay)
GPIO MODE 17, "OUTPUT"

' Set pin 27 as input (for button, sensor)
GPIO MODE 27, "INPUT"

' Set input with pull-up resistor
GPIO MODE 27, "INPUT_PULLUP"

' Set input with pull-down resistor
GPIO MODE 22, "INPUT_PULLDOWN"
```

### GPIO SET

Set the state of an output pin.

```bas
' Turn on LED (HIGH = 3.3V)
GPIO SET 17, HIGH

' Turn off LED (LOW = 0V)
GPIO SET 17, LOW

' Using numeric values (1 = HIGH, 0 = LOW)
GPIO SET 17, 1
GPIO SET 17, 0
```

### GPIO GET

Read the state of an input pin.

```bas
' Read button state
GPIO MODE 27, "INPUT_PULLUP"
state = GPIO GET 27

IF state = LOW THEN
    TALK "Button pressed!"
END IF
```

### GPIO PWM

Generate PWM signal for motor speed, LED brightness.

```bas
' Set PWM on pin 18, duty cycle 0-100
GPIO PWM 18, 50    ' 50% duty cycle

' Fade LED
FOR brightness = 0 TO 100 STEP 5
    GPIO PWM 18, brightness
    WAIT 0.05
NEXT

' Stop PWM
GPIO PWM 18, 0
```

### GPIO SERVO

Control servo motors (uses hardware PWM).

```bas
' Move servo to angle (0-180 degrees)
GPIO SERVO 12, 90    ' Center position
GPIO SERVO 12, 0     ' Left position
GPIO SERVO 12, 180   ' Right position
```

## I2C Keywords

For sensors and displays connected via I2C bus.

### I2C SCAN

Scan for connected I2C devices.

```bas
' Scan I2C bus 1 (default on Pi)
devices = I2C SCAN 1

FOR EACH addr IN devices
    TALK "Found device at address: " + HEX(addr)
NEXT
```

### I2C READ

Read data from an I2C device.

```bas
' Read temperature from sensor at address 0x48
' Register 0x00 contains temperature
temp_raw = I2C READ 0x48, 0x00, 2    ' Read 2 bytes

' Convert to temperature
temperature = temp_raw / 256.0
TALK "Temperature: " + temperature + "°C"
```

### I2C WRITE

Write data to an I2C device.

```bas
' Write configuration to sensor
I2C WRITE 0x48, 0x01, 0x60    ' Write 0x60 to register 0x01

' Write multiple bytes
I2C WRITE 0x3C, 0x00, [0xAE, 0xD5, 0x80]    ' OLED init sequence
```

## SPI Keywords

For high-speed communication with displays and sensors.

### SPI OPEN

Open SPI device.

```bas
' Open SPI device 0, chip select 0
SPI OPEN 0, 0, 1000000    ' 1MHz clock speed
```

### SPI TRANSFER

Send and receive data.

```bas
' Send command to display
response = SPI TRANSFER [0x9F]    ' Read ID command

' Send data to display
SPI TRANSFER [0x00, 0xFF, 0x00, 0xFF]    ' Pattern data
```

### SPI CLOSE

Close SPI connection.

```bas
SPI CLOSE 0, 0
```

## Sensor Keywords

High-level keywords for common sensors.

### READ TEMPERATURE

Read temperature from common sensors (auto-detects DHT11, DHT22, DS18B20, BME280).

```bas
temp = READ TEMPERATURE 4    ' GPIO pin 4
TALK "Temperature: " + temp + "°C"
```

### READ HUMIDITY

Read humidity from DHT11/DHT22/BME280.

```bas
humidity = READ HUMIDITY 4
TALK "Humidity: " + humidity + "%"
```

### READ DISTANCE

Read distance from ultrasonic sensor (HC-SR04).

```bas
' Trigger pin 23, Echo pin 24
distance = READ DISTANCE 23, 24
TALK "Distance: " + distance + " cm"
```

### READ MOTION

Read PIR motion sensor.

```bas
GPIO MODE 7, "INPUT"
motion = READ MOTION 7

IF motion THEN
    TALK "Motion detected!"
    GPIO SET 17, HIGH    ' Turn on light
END IF
```

### READ LIGHT

Read light level from photoresistor or light sensor.

```bas
light = READ LIGHT 0    ' ADC channel 0 (MCP3008)
TALK "Light level: " + light
```

## Display Keywords

### LCD INIT

Initialize character LCD (HD44780).

```bas
' I2C LCD at address 0x27, 16 columns, 2 rows
LCD INIT 0x27, 16, 2
```

### LCD PRINT

Print text to LCD.

```bas
LCD PRINT "Hello World!"

' Print at specific position (row, column)
LCD PRINT 0, 0, "Line 1"
LCD PRINT 1, 0, "Line 2"
```

### LCD CLEAR

Clear the LCD display.

```bas
LCD CLEAR
```

### OLED INIT

Initialize OLED display (SSD1306).

```bas
' I2C OLED at address 0x3C, 128x64 pixels
OLED INIT 0x3C, 128, 64
```

### OLED PRINT

Print text to OLED.

```bas
OLED PRINT "Hello!"

' With position and size
OLED PRINT 0, 0, "Title", 2    ' Size 2x
OLED PRINT 0, 20, "Subtitle", 1
```

### OLED DRAW

Draw shapes on OLED.

```bas
' Draw rectangle
OLED DRAW "rect", 10, 10, 50, 30

' Draw circle
OLED DRAW "circle", 64, 32, 20

' Draw line
OLED DRAW "line", 0, 0, 127, 63
```

### OLED CLEAR

Clear the OLED display.

```bas
OLED CLEAR
```

## Relay & Actuator Keywords

### RELAY SET

Control relay module.

```bas
' Turn on relay 1 (pin 17)
RELAY SET 17, ON

' Turn off relay
RELAY SET 17, OFF

' Toggle relay
state = GPIO GET 17
RELAY SET 17, NOT state
```

### BUZZER

Control buzzer for alerts.

```bas
' Beep for 0.5 seconds
BUZZER 18, 0.5

' Play tone (frequency, duration)
BUZZER 18, 1000, 0.2    ' 1000Hz for 0.2 seconds

' Play melody
BUZZER 18, 262, 0.25    ' C4
BUZZER 18, 294, 0.25    ' D4
BUZZER 18, 330, 0.25    ' E4
```

## Complete Examples

### Smart Doorbell

```bas
' Smart doorbell with camera and notification
' Hardware: Button on GPIO 27, Buzzer on GPIO 18, LED on GPIO 17

GPIO MODE 27, "INPUT_PULLUP"
GPIO MODE 18, "OUTPUT"
GPIO MODE 17, "OUTPUT"

TALK "Doorbell system ready"
LCD PRINT "Ring the bell"

mainLoop:
    button = GPIO GET 27
    
    IF button = LOW THEN
        ' Button pressed
        GPIO SET 17, HIGH        ' LED on
        BUZZER 18, 1000, 0.5     ' Ring buzzer
        
        ' Take photo (if camera connected)
        photo = CAPTURE PHOTO
        
        ' Send notification
        SEND MAIL "owner@home.com", "Doorbell", "Someone at the door!", photo
        
        LCD PRINT "Visitor!"
        WAIT 2
        LCD PRINT "Ring the bell"
        GPIO SET 17, LOW         ' LED off
    END IF
    
    WAIT 0.1
GOTO mainLoop
```

### Temperature Monitor

```bas
' Temperature and humidity monitor with OLED display
' Hardware: DHT22 on GPIO 4, OLED on I2C 0x3C

OLED INIT 0x3C, 128, 64

mainLoop:
    temp = READ TEMPERATURE 4
    humidity = READ HUMIDITY 4
    
    OLED CLEAR
    OLED PRINT 0, 0, "Temperature:", 1
    OLED PRINT 0, 15, temp + " C", 2
    OLED PRINT 0, 40, "Humidity:", 1
    OLED PRINT 0, 55, humidity + " %", 2
    
    ' Alert if too hot
    IF temp > 30 THEN
        BUZZER 18, 2000, 0.1
        SEND MAIL "admin@home.com", "Alert", "Temperature high: " + temp + "C"
    END IF
    
    WAIT 5    ' Update every 5 seconds
GOTO mainLoop
```

### Light Automation

```bas
' Automatic light control based on motion and ambient light
' Hardware: PIR on GPIO 7, Light sensor on ADC0, Relay on GPIO 17

GPIO MODE 7, "INPUT"
GPIO MODE 17, "OUTPUT"

threshold = 500    ' Light threshold
timeout = 30       ' Seconds to keep light on

lastMotion = 0

mainLoop:
    motion = READ MOTION 7
    light = READ LIGHT 0
    
    IF motion AND light < threshold THEN
        ' Dark and motion detected
        RELAY SET 17, ON
        lastMotion = NOW
        LCD PRINT "Light ON"
    END IF
    
    ' Turn off after timeout
    IF NOW - lastMotion > timeout THEN
        RELAY SET 17, OFF
        LCD PRINT "Light OFF"
    END IF
    
    WAIT 0.5
GOTO mainLoop
```

### Voice-Controlled Lights

```bas
' Voice control for home automation
' Ask the AI to control lights

TALK "What would you like me to do?"
command = HEAR

' Use LLM to understand command
intent = LLM "Extract intent from: '" + command + "'. Return JSON: {action: on/off, room: string}"
intent = JSON PARSE intent

SELECT CASE intent.room
    CASE "living room"
        RELAY SET 17, intent.action = "on"
    CASE "bedroom"
        RELAY SET 18, intent.action = "on"
    CASE "kitchen"
        RELAY SET 27, intent.action = "on"
    CASE "all"
        RELAY SET 17, intent.action = "on"
        RELAY SET 18, intent.action = "on"
        RELAY SET 27, intent.action = "on"
END SELECT

TALK "Done! " + intent.room + " light is now " + intent.action
```

## Pin Reference

### Raspberry Pi GPIO

| Pin | GPIO | Common Use |
|-----|------|------------|
| 11 | GPIO17 | Output (LED, Relay) |
| 12 | GPIO18 | PWM (Motor, Servo) |
| 13 | GPIO27 | Input (Button) |
| 15 | GPIO22 | Input/Output |
| 16 | GPIO23 | Input/Output |
| 18 | GPIO24 | Input/Output |
| 3 | SDA | I2C Data |
| 5 | SCL | I2C Clock |
| 19 | MOSI | SPI Data Out |
| 21 | MISO | SPI Data In |
| 23 | SCLK | SPI Clock |

### Orange Pi GPIO

Similar pinout, but check your specific model. Orange Pi 5 uses different GPIO numbering.

## Troubleshooting

### Permission Denied

```bash
# Add user to gpio group
sudo usermod -a -G gpio $USER
sudo usermod -a -G i2c $USER
sudo usermod -a -G spi $USER

# Or run botserver with elevated privileges
sudo systemctl edit botserver
# Add: User=root
```

### I2C Not Working

```bash
# Enable I2C
sudo raspi-config  # Interface Options → I2C

# Check if device detected
i2cdetect -y 1
```

### GPIO Busy

```bash
# Check what's using the GPIO
cat /sys/kernel/debug/gpio

# Release GPIO
echo 17 > /sys/class/gpio/unexport
