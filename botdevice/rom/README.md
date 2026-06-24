# BotOS ROM - Sistema Android Customizado

## Níveis de Customização

### Nível 1: Debloat + Launcher (Sem Root)
Remove apps do fabricante via ADB e configura BotOS como launcher padrão.

### Nível 2: Magisk Module (Com Root)
Módulo que substitui boot animation, launcher padrão, e remove bloatware permanentemente.

### Nível 3: Custom GSI (Imagem Completa)
Android puro com BotOS integrado, sem nenhum app de fabricante.

---

## Nível 1: Debloat sem Root

```bash
# Conectar dispositivo via USB (debug USB ativado)
./scripts/debloat.sh
```

Remove apps Samsung, Huawei, Xiaomi sem precisar de root.

## Nível 2: Magisk Module

```bash
# Gerar módulo Magisk
./scripts/build-magisk-module.sh

# Instalar via Magisk Manager
adb push botos-magisk.zip /sdcard/
```

## Nível 3: GSI Build

Requer ambiente de build AOSP. Veja `gsi/README.md`.

---

## Fabricantes Suportados para Debloat

- Samsung (One UI)
- Huawei (EMUI)
- Xiaomi (MIUI)
- Motorola
- LG
- Realme/OPPO (ColorOS)
- Vivo (FuntouchOS)

## Requisitos

- ADB instalado
- USB Debug ativado no dispositivo
- Para Nível 2+: Bootloader desbloqueado + Magisk
- Para Nível 3: Ambiente AOSP build (~200GB espaço)
