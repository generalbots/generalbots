#!/bin/bash
# Build BotOS Magisk Module
# Substitui boot animation, launcher padrão, e configurações do sistema
#
# O módulo será instalado via Magisk Manager

set -e
cd "$(dirname "$0")/.."

MODULE_DIR="magisk-module"
MODULE_ZIP="botos-magisk-v1.0.zip"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           Building BotOS Magisk Module                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Limpar e criar estrutura
rm -rf "$MODULE_DIR"
mkdir -p "$MODULE_DIR/META-INF/com/google/android"
mkdir -p "$MODULE_DIR/system/media"
mkdir -p "$MODULE_DIR/system/etc/permissions"
mkdir -p "$MODULE_DIR/system/priv-app/BotOS"
mkdir -p "$MODULE_DIR/common"

# =====================================================
# module.prop - Metadados do módulo
# =====================================================
cat > "$MODULE_DIR/module.prop" << 'EOF'
id=botos
name=BotOS - General Bots Launcher
version=v1.0
versionCode=1
author=Pragmatismo.io
description=Transforma seu Android em BotOS - Remove bloatware, substitui boot animation e define BotOS como launcher padrão
EOF

# =====================================================
# update-binary - Script de instalação
# =====================================================
cat > "$MODULE_DIR/META-INF/com/google/android/update-binary" << 'INSTALLER'
#!/sbin/sh

#################
# Initialization
#################

TMPDIR=/dev/tmp
PERSISTDIR=/sbin/.magisk/mirror/persist

rm -rf $TMPDIR 2>/dev/null
mkdir -p $TMPDIR

# echo before loading util_functions
ui_print() { echo "$1"; }

require_new_magisk() {
  ui_print "*******************************"
  ui_print " Please install Magisk v20.4+! "
  ui_print "*******************************"
  exit 1
}

##########################
# Load util_functions.sh
##########################

OUTFD=$2
ZIPFILE=$3

mount /data 2>/dev/null

[ -f /data/adb/magisk/util_functions.sh ] || require_new_magisk
. /data/adb/magisk/util_functions.sh
[ $MAGISK_VER_CODE -lt 20400 ] && require_new_magisk

install_module
exit 0
INSTALLER

chmod 755 "$MODULE_DIR/META-INF/com/google/android/update-binary"

# =====================================================
# updater-script (vazio, necessário para compatibilidade)
# =====================================================
echo "#MAGISK" > "$MODULE_DIR/META-INF/com/google/android/updater-script"

# =====================================================
# customize.sh - Script de customização pós-instalação
# =====================================================
cat > "$MODULE_DIR/customize.sh" << 'CUSTOMIZE'
#!/system/bin/sh
# BotOS Magisk Module - Customization Script

ui_print "╔══════════════════════════════════════════════════════════════╗"
ui_print "║                 Installing BotOS                             ║"
ui_print "╚══════════════════════════════════════════════════════════════╝"
ui_print ""

# Verificar arquitetura
ARCH=$(getprop ro.product.cpu.abi)
ui_print "- Device architecture: $ARCH"
ui_print "- Android version: $(getprop ro.build.version.release)"
ui_print "- Device: $(getprop ro.product.model)"
ui_print ""

# Copiar boot animation se existir
if [ -f "$MODPATH/bootanimation.zip" ]; then
    ui_print "- Installing custom boot animation..."
    mkdir -p "$MODPATH/system/media"
    cp "$MODPATH/bootanimation.zip" "$MODPATH/system/media/"
fi

# Copiar APK do BotOS se existir
if [ -f "$MODPATH/BotOS.apk" ]; then
    ui_print "- Installing BotOS launcher as system app..."
    mkdir -p "$MODPATH/system/priv-app/BotOS"
    cp "$MODPATH/BotOS.apk" "$MODPATH/system/priv-app/BotOS/"
fi

# Configurar permissões
ui_print "- Setting permissions..."
set_perm_recursive $MODPATH 0 0 0755 0644

if [ -d "$MODPATH/system/priv-app" ]; then
    set_perm_recursive "$MODPATH/system/priv-app" 0 0 0755 0644
fi

ui_print ""
ui_print "╔══════════════════════════════════════════════════════════════╗"
ui_print "║              BotOS installed successfully!                   ║"
ui_print "║                                                              ║"
ui_print "║  After reboot:                                               ║"
ui_print "║  1. Press Home button                                        ║"
ui_print "║  2. Select 'BotOS' as default launcher                       ║"
ui_print "║  3. Choose 'Always'                                          ║"
ui_print "╚══════════════════════════════════════════════════════════════╝"
CUSTOMIZE

chmod 755 "$MODULE_DIR/customize.sh"

# =====================================================
# post-fs-data.sh - Executado no boot (após fs montado)
# =====================================================
cat > "$MODULE_DIR/post-fs-data.sh" << 'POSTFS'
#!/system/bin/sh
# BotOS - Post-FS-Data Script
# Executado após o sistema de arquivos ser montado

MODDIR=${0%/*}

# Log
log -t BotOS "BotOS module post-fs-data started"

# Desabilitar apps de fabricante (overlay)
# Isso "esconde" os apps sem deletá-los
for bloat in \
    "com.facebook.katana" \
    "com.facebook.appmanager" \
    "com.facebook.services" \
    "com.samsung.android.bixby.agent" \
    "com.samsung.android.bixby.service" \
    "com.huawei.appmarket" \
    "com.miui.msa.global" \
    "com.miui.analytics"
do
    # Criar diretório vazio para "substituir" o app
    if [ -d "/system/app/$bloat" ]; then
        mkdir -p "$MODDIR/system/app/$bloat"
        touch "$MODDIR/system/app/$bloat/.replace"
    fi
    if [ -d "/system/priv-app/$bloat" ]; then
        mkdir -p "$MODDIR/system/priv-app/$bloat"
        touch "$MODDIR/system/priv-app/$bloat/.replace"
    fi
done

log -t BotOS "BotOS bloatware disabled via overlay"
POSTFS

chmod 755 "$MODULE_DIR/post-fs-data.sh"

# =====================================================
# service.sh - Executado como serviço no boot
# =====================================================
cat > "$MODULE_DIR/service.sh" << 'SERVICE'
#!/system/bin/sh
# BotOS - Service Script
# Executado como serviço após o boot completar

MODDIR=${0%/*}

# Aguardar boot completar
while [ "$(getprop sys.boot_completed)" != "1" ]; do
    sleep 1
done

# Aguardar mais um pouco para sistema estabilizar
sleep 5

log -t BotOS "BotOS service started"

# Configurar BotOS como launcher padrão (se instalado)
BOTOS_PKG="br.com.pragmatismo.botos"

if pm list packages | grep -q "$BOTOS_PKG"; then
    # Tentar definir como launcher padrão
    # Nota: Isso pode não funcionar em todos os dispositivos
    pm set-home-activity "$BOTOS_PKG/.MainActivity" 2>/dev/null || true
    log -t BotOS "BotOS set as preferred launcher"
fi

# Desabilitar analytics/telemetria de fabricantes
settings put global upload_apk_enable 0 2>/dev/null || true
settings put secure send_action_app_error 0 2>/dev/null || true

# Ativar modo imersivo (esconde barra de navegação)
# settings put global policy_control immersive.full=* 2>/dev/null || true

log -t BotOS "BotOS service configuration complete"
SERVICE

chmod 755 "$MODULE_DIR/service.sh"

# =====================================================
# uninstall.sh - Script de desinstalação
# =====================================================
cat > "$MODULE_DIR/uninstall.sh" << 'UNINSTALL'
#!/system/bin/sh
# BotOS - Uninstall Script

# Restaurar launcher padrão do sistema
# (O Magisk automaticamente remove os overlays)

log -t BotOS "BotOS module uninstalled"
UNINSTALL

chmod 755 "$MODULE_DIR/uninstall.sh"

# =====================================================
# Copiar boot animation se existir
# =====================================================
if [ -f "../bootanimation.zip" ]; then
    echo "- Incluindo boot animation customizada..."
    cp "../bootanimation.zip" "$MODULE_DIR/"
else
    echo "- Boot animation não encontrada."
    echo "  Execute ../scripts/create-bootanimation.sh primeiro."
fi

# =====================================================
# Copiar APK do BotOS se existir
# =====================================================
APK_PATH="../gen/android/app/build/outputs/apk/release/app-release.apk"
if [ -f "$APK_PATH" ]; then
    echo "- Incluindo BotOS APK..."
    cp "$APK_PATH" "$MODULE_DIR/BotOS.apk"
else
    echo "- APK não encontrado."
    echo "  Compile com: cd .. && cargo tauri android build --release"
fi

# =====================================================
# Criar arquivo ZIP do módulo
# =====================================================
echo ""
echo "Criando $MODULE_ZIP..."
cd "$MODULE_DIR"
zip -r "../$MODULE_ZIP" . -x "*.DS_Store"
cd ..

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           Magisk Module criado com sucesso!                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Arquivo: $(pwd)/$MODULE_ZIP"
echo ""
echo "Para instalar:"
echo "  1. Copie para o dispositivo:"
echo "     adb push $MODULE_ZIP /sdcard/"
echo ""
echo "  2. Abra o Magisk Manager"
echo "  3. Vá em 'Modules' → '+' → Selecione o ZIP"
echo "  4. Reinicie o dispositivo"
echo ""
