#!/bin/bash
# BotOS Debloat Script - Remove bloatware sem root
# Funciona via ADB shell pm uninstall --user 0
#
# Uso: ./debloat.sh [samsung|huawei|xiaomi|all]

set -e
cd "$(dirname "$0")/.."

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              BotOS Debloat Tool v1.0                         ║"
echo "║      Remove bloatware Samsung/Huawei/Xiaomi/etc              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Verificar ADB
if ! command -v adb &> /dev/null; then
    echo -e "${RED}ERRO: ADB não encontrado!${NC}"
    echo "Instale com: sudo apt install adb"
    exit 1
fi

# Verificar dispositivo conectado
if ! adb devices | grep -q "device$"; then
    echo -e "${RED}ERRO: Nenhum dispositivo conectado!${NC}"
    echo "1. Ative 'Depuração USB' nas configurações do desenvolvedor"
    echo "2. Conecte o cabo USB e autorize no celular"
    exit 1
fi

DEVICE=$(adb shell getprop ro.product.model | tr -d '\r')
BRAND=$(adb shell getprop ro.product.brand | tr -d '\r')
echo -e "${GREEN}Dispositivo detectado: $BRAND $DEVICE${NC}"
echo ""

# =====================================================
# LISTAS DE BLOATWARE POR FABRICANTE
# =====================================================

# Samsung One UI
SAMSUNG_BLOAT=(
    # Samsung Apps
    "com.samsung.android.app.tips"
    "com.samsung.android.bixby.agent"
    "com.samsung.android.bixby.service"
    "com.samsung.android.visionintelligence"
    "com.samsung.android.app.routines"
    "com.samsung.android.game.gamehome"
    "com.samsung.android.game.gametools"
    "com.samsung.android.app.spage"
    "com.samsung.android.mateagent"
    "com.samsung.android.app.watchmanagerstub"
    "com.samsung.android.ardrawing"
    "com.samsung.android.aremoji"
    "com.samsung.android.arzone"
    "com.samsung.android.stickercenter"
    "com.samsung.android.app.dressroom"
    "com.samsung.android.forest"
    "com.samsung.android.app.social"
    "com.samsung.android.livestickers"
    "com.samsung.android.app.sharelive"
    
    # Samsung Duplicates (use Google apps instead)
    "com.samsung.android.email.provider"
    "com.samsung.android.calendar"
    "com.samsung.android.contacts"
    "com.samsung.android.messaging"
    "com.sec.android.app.sbrowser"
    
    # Facebook bloatware (pre-installed)
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
    "com.facebook.system"
    
    # Microsoft bloatware
    "com.microsoft.skydrive"
    "com.microsoft.office.excel"
    "com.microsoft.office.word"
    "com.microsoft.office.powerpoint"
    "com.linkedin.android"
    
    # Other Samsung bloat
    "com.samsung.android.spay"
    "com.samsung.android.samsungpass"
    "com.samsung.android.authfw"
    "com.samsung.android.kidsinstaller"
    "com.samsung.android.app.camera.sticker.facearavatar.preload"
)

# Huawei EMUI
HUAWEI_BLOAT=(
    "com.huawei.hiview"
    "com.huawei.himovie.overseas"
    "com.huawei.music"
    "com.huawei.appmarket"
    "com.huawei.browser"
    "com.huawei.hifolder"
    "com.huawei.gameassistant"
    "com.huawei.tips"
    "com.huawei.hwid"
    "com.huawei.wallet"
    "com.huawei.health"
    "com.huawei.hicloud"
    "com.huawei.compass"
    "com.huawei.mirrorlink"
    "com.huawei.hicar"
    "com.huawei.hiai"
    "com.huawei.intelligent"
    "com.huawei.parentcontrol"
    "com.huawei.securitymgr"
    
    # Facebook
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
    
    # Booking
    "com.booking"
)

# Xiaomi MIUI
XIAOMI_BLOAT=(
    "com.miui.analytics"
    "com.miui.msa.global"
    "com.miui.daemon"
    "com.miui.hybrid"
    "com.miui.yellowpage"
    "com.miui.videoplayer"
    "com.miui.player"
    "com.miui.compass"
    "com.miui.cleanmaster"
    "com.miui.gallery"
    "com.miui.weather2"
    "com.miui.notes"
    "com.miui.calculator"
    "com.miui.mishare.connectivity"
    "com.xiaomi.glgm"
    "com.xiaomi.joyose"
    "com.xiaomi.mipicks"
    "com.xiaomi.midrop"
    "com.mi.android.globalminusscreen"
    "com.mi.android.globallauncher"
    "com.mi.health"
    
    # Games
    "com.miui.bugreport"
    "cn.wps.xiaomi.abroad.lite"
    
    # Facebook
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
    "com.facebook.system"
)

# Oppo/Realme ColorOS
OPPO_BLOAT=(
    "com.coloros.gamespace"
    "com.coloros.weather"
    "com.coloros.compass2"
    "com.coloros.filemanager"
    "com.coloros.floatassistant"
    "com.coloros.gallery3d"
    "com.coloros.video"
    "com.coloros.music"
    "com.coloros.smartdrive"
    "com.heytap.browser"
    "com.heytap.music"
    "com.heytap.cloud"
    "com.oppo.market"
    
    # Facebook
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
)

# Motorola
MOTO_BLOAT=(
    "com.motorola.help"
    "com.motorola.demo"
    "com.motorola.motocare"
    "com.motorola.ccc.mainplm"
    "com.motorola.android.providers.settings"
    "com.motorola.actions"
    "com.motorola.gamemode"
    
    # Facebook
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
)

# Universal bloatware (all vendors)
UNIVERSAL_BLOAT=(
    # Facebook (pre-installed em quase todos)
    "com.facebook.katana"
    "com.facebook.appmanager"
    "com.facebook.services"
    "com.facebook.system"
    "com.facebook.orca"
    
    # Netflix (pre-installed)
    "com.netflix.mediaclient"
    "com.netflix.partner.activation"
    
    # Spotify (pre-installed)
    "com.spotify.music"
    
    # TikTok
    "com.zhiliaoapp.musically"
    "com.ss.android.ugc.trill"
    
    # Games pre-instalados
    "com.king.candycrushsaga"
    "com.gameloft.android.ANMP.GloftA9HM"
)

# =====================================================
# FUNÇÕES
# =====================================================

uninstall_package() {
    local pkg=$1
    echo -n "  Removendo $pkg... "
    
    # Verificar se está instalado
    if adb shell pm list packages | grep -q "^package:$pkg$"; then
        if adb shell pm uninstall -k --user 0 "$pkg" 2>/dev/null | grep -q "Success"; then
            echo -e "${GREEN}OK${NC}"
            return 0
        else
            echo -e "${YELLOW}Falhou (protegido)${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}Não instalado${NC}"
        return 0
    fi
}

debloat_list() {
    local name=$1
    shift
    local packages=("$@")
    
    echo -e "\n${BLUE}=== Removendo $name bloatware ===${NC}\n"
    
    local removed=0
    local failed=0
    
    for pkg in "${packages[@]}"; do
        if uninstall_package "$pkg"; then
            ((removed++))
        else
            ((failed++))
        fi
    done
    
    echo -e "\n${GREEN}$removed removidos${NC}, ${YELLOW}$failed protegidos${NC}"
}

detect_vendor() {
    local brand=$(echo "$BRAND" | tr '[:upper:]' '[:lower:]')
    
    case "$brand" in
        samsung)
            echo "samsung"
            ;;
        huawei|honor)
            echo "huawei"
            ;;
        xiaomi|redmi|poco)
            echo "xiaomi"
            ;;
        oppo|realme|oneplus)
            echo "oppo"
            ;;
        motorola)
            echo "moto"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

install_botos_launcher() {
    echo -e "\n${BLUE}=== Instalando BotOS Launcher ===${NC}\n"
    
    local apk="../gen/android/app/build/outputs/apk/release/app-release.apk"
    
    if [[ -f "$apk" ]]; then
        adb install -r "$apk"
        echo -e "${GREEN}BotOS instalado!${NC}"
        
        # Definir como launcher padrão
        echo "Para definir como launcher padrão:"
        echo "  1. Pressione o botão Home"
        echo "  2. Selecione 'BotOS' na lista"
        echo "  3. Escolha 'Sempre'"
    else
        echo -e "${YELLOW}APK não encontrado. Compile primeiro:${NC}"
        echo "  cd .. && cargo tauri android build --release"
    fi
}

enable_kiosk_mode() {
    echo -e "\n${BLUE}=== Configurando Kiosk Mode ===${NC}\n"
    
    # Desabilitar navegação por gestos
    adb shell settings put global policy_control immersive.full=*
    
    # Esconder barra de navegação
    adb shell settings put global policy_control immersive.navigation=*
    
    echo -e "${GREEN}Kiosk mode ativado!${NC}"
    echo "O usuário não poderá sair do app facilmente."
}

# =====================================================
# MAIN
# =====================================================

VENDOR_ARG=${1:-auto}

if [[ "$VENDOR_ARG" == "auto" ]]; then
    VENDOR=$(detect_vendor)
    echo -e "Fabricante detectado: ${GREEN}$VENDOR${NC}"
else
    VENDOR=$VENDOR_ARG
fi

echo ""
echo "Opções:"
echo "  1) Debloat leve (apenas Facebook/bloat universal)"
echo "  2) Debloat médio (+ apps do fabricante duplicados)"
echo "  3) Debloat agressivo (remove TUDO do fabricante)"
echo "  4) Instalar BotOS e configurar launcher"
echo "  5) Ativar Kiosk Mode (travar no BotOS)"
echo "  6) Executar tudo (2 + 4 + 5)"
echo ""
read -p "Escolha [1-6]: " CHOICE

case $CHOICE in
    1)
        debloat_list "Universal" "${UNIVERSAL_BLOAT[@]}"
        ;;
    2)
        debloat_list "Universal" "${UNIVERSAL_BLOAT[@]}"
        case $VENDOR in
            samsung) debloat_list "Samsung" "${SAMSUNG_BLOAT[@]}" ;;
            huawei) debloat_list "Huawei" "${HUAWEI_BLOAT[@]}" ;;
            xiaomi) debloat_list "Xiaomi" "${XIAOMI_BLOAT[@]}" ;;
            oppo) debloat_list "Oppo/Realme" "${OPPO_BLOAT[@]}" ;;
            moto) debloat_list "Motorola" "${MOTO_BLOAT[@]}" ;;
        esac
        ;;
    3)
        debloat_list "Universal" "${UNIVERSAL_BLOAT[@]}"
        debloat_list "Samsung" "${SAMSUNG_BLOAT[@]}"
        debloat_list "Huawei" "${HUAWEI_BLOAT[@]}"
        debloat_list "Xiaomi" "${XIAOMI_BLOAT[@]}"
        debloat_list "Oppo/Realme" "${OPPO_BLOAT[@]}"
        debloat_list "Motorola" "${MOTO_BLOAT[@]}"
        ;;
    4)
        install_botos_launcher
        ;;
    5)
        enable_kiosk_mode
        ;;
    6)
        debloat_list "Universal" "${UNIVERSAL_BLOAT[@]}"
        case $VENDOR in
            samsung) debloat_list "Samsung" "${SAMSUNG_BLOAT[@]}" ;;
            huawei) debloat_list "Huawei" "${HUAWEI_BLOAT[@]}" ;;
            xiaomi) debloat_list "Xiaomi" "${XIAOMI_BLOAT[@]}" ;;
            oppo) debloat_list "Oppo/Realme" "${OPPO_BLOAT[@]}" ;;
            moto) debloat_list "Motorola" "${MOTO_BLOAT[@]}" ;;
        esac
        install_botos_launcher
        enable_kiosk_mode
        ;;
    *)
        echo "Opção inválida"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    Debloat concluído!                        ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Reinicie o dispositivo para aplicar todas as mudanças:"
echo "  adb reboot"
