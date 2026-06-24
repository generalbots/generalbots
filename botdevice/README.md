# BotDevice - Android, HarmonyOS & IoT powered by General Bots

**BotDevice** transforms any Android, HarmonyOS, or embedded device into a dedicated General Bots system, removing all manufacturer bloatware (Samsung, Huawei, Xiaomi, Honor, etc) and replacing it with the GB interface.

## Supported Platforms

### Mobile
- **Android** (AOSP, Samsung One UI, Xiaomi MIUI, etc)
- **HarmonyOS** (Huawei, Honor)

### Embedded / IoT
- **Raspberry Pi** (Zero, 3, 4, 5) - Linux with LCD/HDMI display
- **Orange Pi** - Budget Raspberry alternative with NPU
- **Banana Pi** - ARM boards with display
- **BeagleBone** - Industrial IoT
- **Arduino** (with ESP32/ESP8266) - OLED/LCD display + WiFi
- **ESP32** - TFT/OLED displays
- **Rock Pi** - RK3399/RK3588 boards
- **NVIDIA Jetson** - Edge AI with display
- **LattePanda** - x86 embedded
- **ODROID** - Hardkernel boards

### Supported Displays
- LCD Character (16x2, 20x4)
- OLED (128x64, 128x32)
- TFT/IPS (320x240, 480x320, 800x480)
- E-ink/E-paper
- HDMI (any resolution)

## Installation Levels

| Level | Requirements | What it does |
|-------|-------------|--------------|
| **1** | ADB only | Removes bloatware, installs BotDevice as app |
| **2** | Root + Magisk | GB boot animation, BotDevice as system app |
| **3** | Unlocked bootloader | Replaces entire Android with BotDevice |

## Quick Start

```bash
cd botdevice/rom
./install.sh
```

The installer automatically detects the device and shows available options.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LEVEL 3: GSI                                    │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Custom Android AOSP - Zero manufacturer apps                            ││
│  │ GB boot animation from startup                                          ││
│  │ BotDevice integrated as single launcher                                 ││
│  └─────────────────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────────────┤
│                           LEVEL 2: MAGISK MODULE                            │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Original Android + Magisk                                               ││
│  │ Bloatware removed via overlay                                           ││
│  │ GB boot animation                                                       ││
│  │ BotDevice as privileged system app                                      ││
│  └─────────────────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────────────┤
│                          LEVEL 1: DEBLOAT + APP                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Original Android (Samsung/Huawei/Xiaomi/etc)                            ││
│  │ Bloatware removed via ADB (no root)                                     ││
│  │ BotDevice installed as normal app                                       ││
│  │ Can be set as default launcher                                          ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           BotDevice App (Tauri)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  botui/ui/suite        │  Tauri Android     │  src/lib.rs (Rust)           │
│  (Web Interface)       │  (WebView + NDK)   │  (Backend + Hardware)        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
botdevice/
├── Cargo.toml                    # Rust/Tauri dependencies
├── tauri.conf.json               # Tauri config → botui/ui/suite
├── build.rs                      # Build script
├── src/lib.rs                    # Android entry point
│
├── icons/
│   ├── gb-bot.svg                # Source icon
│   ├── icon.png (512x512)        # Main icon
│   └── */ic_launcher.png         # Icons by density
│
├── scripts/
│   ├── generate-icons.sh         # Generate PNGs from SVG
│   └── create-bootanimation.sh   # Generate boot animation
│
├── capabilities/
│   └── default.json              # Tauri permissions
│
├── gen/android/                  # Generated Android project
│   └── app/src/main/
│       ├── AndroidManifest.xml   # HOME intent (launcher)
│       └── res/values/themes.xml # GB dark theme
│
└── rom/                          # Installation tools
    ├── install.sh                # Interactive installer
    ├── scripts/
    │   ├── debloat.sh            # Remove bloatware (no root)
    │   └── build-magisk-module.sh # Generate Magisk module
    └── gsi/
        ├── README.md             # GSI/AOSP instructions
        └── device/pragmatismo/botdevice/  # AOSP device tree
```

## Prerequisites

### To compile BotDevice App

```bash
# Rust and Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi

# Android SDK and NDK
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653

# Tauri CLI
cargo install tauri-cli
```

### To install on devices

```bash
# ADB
sudo apt install adb

# To generate icons/boot animation
sudo apt install librsvg2-bin imagemagick
```

## Building

```bash
cd botdevice

# Generate icons
./scripts/generate-icons.sh

# Initialize Android project
cargo tauri android init

# Build APK
cargo tauri android build --release
```

## Installation

### Quick Method (Interactive)

```bash
cd botdevice/rom
chmod +x install.sh
./install.sh
```

### Manual Method

#### Level 1: Debloat + App (No Root)

```bash
# Connect device via USB (debug enabled)
cd botdevice/rom/scripts
./debloat.sh

# Install APK
adb install ../gen/android/app/build/outputs/apk/release/app-release.apk

# Set as launcher: Home → BotDevice → Always
```

#### Level 2: Magisk Module (With Root)

```bash
# Generate module
cd botdevice/rom/scripts
./build-magisk-module.sh

# Copy to device
adb push botdevice-magisk-v1.0.zip /sdcard/

# On phone: Magisk → Modules → + → Select ZIP → Reboot
```

#### Level 3: GSI (Unlocked Bootloader)

See detailed instructions in `rom/gsi/README.md`.

## Bloatware Removed

The debloat automatically removes:

**Samsung One UI:**
- Bixby, Samsung Pay, Samsung Pass
- Duplicate apps (Email, Calendar, Browser)
- AR Zone, Game Launcher

**Huawei EMUI/HarmonyOS:**
- AppGallery, HiCloud, HiCar
- Huawei Browser, Music, Video
- Petal Maps, Petal Search
- AI Life, HiSuite

**Honor MagicOS:**
- Honor Store, MagicRing
- Honor Browser, Music

**Xiaomi MIUI:**
- MSA (analytics), Mi Apps
- GetApps, Mi Cloud

**Universal (all):**
- Pre-installed Facebook, Instagram
- Pre-installed Netflix, Spotify
- Games like Candy Crush

## Boot Animation

To customize the boot animation (requires root):

```bash
# Generate animation
./scripts/create-bootanimation.sh

# Install (root)
adb root
adb remount
adb push bootanimation.zip /system/media/
adb reboot
```

## Development

```bash
# Dev mode (connects to device)
cargo tauri android dev

# Logs
adb logcat -s BotDevice:*
```

## Embedded Interface (LCD/Keyboard)

For devices with limited resources, use the embedded interface at `botui/ui/embedded/`:

```bash
# Raspberry Pi with LCD display
chromium-browser --kiosk --app=http://localhost:9000/embedded/

# ESP32 with TFT display (via WebView)
# Configure BOTSERVER_URL in firmware

# Character terminal mode
# Use botui/ui/embedded/ with CONFIG.maxMsgLen adjusted
```

### Embedded Interface Features
- Optimized for displays 320x240 down to 16x2 characters
- High contrast (green/black, e-ink)
- Low memory usage (max 10 messages)
- Keyboard navigation (Enter sends, Esc clears)
- Auto reconnection

## Features

- 🏠 **Launcher Mode**: Replaces home screen
- 🤖 **Chat Interface**: botui/ui/suite
- 🦀 **Rust Backend**: Via Tauri
- 📍 **GPS**: Location access
- 📷 **Camera**: Via Tauri plugins
- 🔔 **Notifications**: Push notifications
- 🌐 **Internet**: Communication with botserver
- 🎨 **Boot Animation**: Customizable with gb-bot.svg

## License

AGPL-3.0
