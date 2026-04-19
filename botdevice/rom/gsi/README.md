# BotOS GSI - Generic System Image

## O que é GSI?

GSI (Generic System Image) é uma imagem Android pura que funciona em **qualquer dispositivo** com Project Treble (Android 8.0+). 

Com GSI você pode:
- Substituir COMPLETAMENTE o sistema do fabricante
- Ter Android puro + BotOS
- Zero bloatware Samsung/Huawei/Xiaomi
- Controle total da inicialização

## Requisitos do Dispositivo

1. **Project Treble** - Verificar com:
   ```bash
   adb shell getprop ro.treble.enabled
   # Deve retornar "true"
   ```

2. **Bootloader desbloqueado**

3. **Partição system tipo A/B ou A-only**:
   ```bash
   adb shell getprop ro.build.ab_update
   # "true" = A/B, vazio = A-only
   ```

## Opções de GSI Base

### Opção 1: Usar GSI existente + BotOS Magisk Module

Mais simples. Use uma GSI pronta e instale o módulo BotOS:

1. Baixe GSI de: https://github.com/nicograph/nicograph-gsi
2. Flash via fastboot
3. Instale Magisk
4. Instale botos-magisk.zip

### Opção 2: Compilar AOSP com BotOS integrado

Para controle total, compile o AOSP do zero.

## Compilando AOSP com BotOS

### Requisitos de Build

```bash
# Ubuntu 20.04+ recomendado
# ~300GB de espaço em disco
# 16GB+ RAM
# CPU com muitos cores (compilação leva horas)

# Instalar dependências
sudo apt install -y git-core gnupg flex bison build-essential \
    zip curl zlib1g-dev libc6-dev-i386 libncurses5 \
    x11proto-core-dev libx11-dev lib32z1-dev libgl1-mesa-dev \
    libxml2-utils xsltproc unzip fontconfig python3
```

### Estrutura do Projeto

```
aosp/
├── .repo/                    # Repo tool metadata
├── build/                    # Build system
├── device/
│   └── pragmatismo/
│       └── botos/            # Device config para BotOS
├── packages/
│   └── apps/
│       └── BotOS/            # App BotOS como system app
├── vendor/
│   └── pragmatismo/
│       └── botos/            # Vendor customizations
└── out/                      # Build output
```

### Passo 1: Inicializar Repo

```bash
mkdir aosp && cd aosp

# Inicializar com Android 14 (ou versão desejada)
repo init -u https://android.googlesource.com/platform/manifest -b android-14.0.0_r1

# Sync (demora muito!)
repo sync -c -j$(nproc) --no-tags
```

### Passo 2: Criar Device Tree BotOS

O device tree define configurações específicas do BotOS.

Veja os arquivos em `device/pragmatismo/botos/` neste repositório.

### Passo 3: Compilar

```bash
source build/envsetup.sh
lunch botos_arm64-userdebug
make -j$(nproc)
```

### Passo 4: Flash

```bash
# Entrar em fastboot mode
adb reboot fastboot

# Flash GSI
fastboot flash system out/target/product/botos/system.img
fastboot flash vendor out/target/product/botos/vendor.img
fastboot flash boot out/target/product/botos/boot.img

# Limpar dados (necessário para GSI)
fastboot -w

# Reboot
fastboot reboot
```

## Arquivos de Configuração

Os arquivos necessários estão em:
- `device/` - Device tree
- `vendor/` - Vendor customizations
- `packages/apps/BotOS/` - App BotOS

## Alternativa: phh-treble GSI

Para uma abordagem mais rápida, modifique o phh-treble GSI:

```bash
git clone https://github.com/nicograph/nicograph-gsi
cd nicograph-gsi

# Adicionar BotOS como system app
mkdir -p nicograph-files/system/priv-app/BotOS
cp /path/to/BotOS.apk nicograph-files/system/priv-app/BotOS/

# Adicionar boot animation
cp /path/to/bootanimation.zip nicograph-files/system/media/

# Rebuild
./build.sh
```

## Notas de Segurança

⚠️ **AVISO**: Modificar system images pode:
- Invalidar garantia do dispositivo
- Causar brick se feito incorretamente
- Desabilitar recursos de segurança (SafetyNet, etc)

Sempre tenha backup completo antes de qualquer modificação!
