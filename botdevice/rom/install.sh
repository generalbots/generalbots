#!/bin/bash
# BotOS Installation Script
# Escolha o nível de instalação baseado no seu dispositivo
#
# Uso: ./install.sh

set -e
cd "$(dirname "$0")"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

clear
echo -e "${CYAN}"
cat << 'BANNER'
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║    ██████╗  ██████╗ ████████╗ ██████╗ ███████╗                              ║
║    ██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝                              ║
║    ██████╔╝██║   ██║   ██║   ██║   ██║███████╗                              ║
║    ██╔══██╗██║   ██║   ██║   ██║   ██║╚════██║                              ║
║    ██████╔╝╚██████╔╝   ██║   ╚██████╔╝███████║                              ║
║    ╚═════╝  ╚═════╝    ╚═╝    ╚═════╝ ╚══════╝                              ║
║                                                                              ║
║                 Android OS by General Bots                                   ║
║                    Pragmatismo.io                                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
BANNER
echo -e "${NC}"

# =====================================================
# Verificações
# =====================================================

check_adb() {
    if ! command -v adb &> /dev/null; then
        echo -e "${RED}ERRO: ADB não encontrado!${NC}"
        echo "Instale com: sudo apt install adb"
        return 1
    fi
    return 0
}

check_device() {
    if ! adb devices | grep -q "device$"; then
        echo -e "${YELLOW}Nenhum dispositivo conectado.${NC}"
        echo ""
        echo "Para conectar:"
        echo "  1. Ative 'Opções do desenvolvedor' no celular"
        echo "     (Configurações → Sobre → Tocar 7x no 'Número da versão')"
        echo "  2. Ative 'Depuração USB'"
        echo "  3. Conecte o cabo USB"
        echo "  4. Autorize no celular quando solicitado"
        echo ""
        return 1
    fi
    
    DEVICE=$(adb shell getprop ro.product.model 2>/dev/null | tr -d '\r')
    BRAND=$(adb shell getprop ro.product.brand 2>/dev/null | tr -d '\r')
    ANDROID=$(adb shell getprop ro.build.version.release 2>/dev/null | tr -d '\r')
    
    echo -e "${GREEN}Dispositivo conectado: $BRAND $DEVICE (Android $ANDROID)${NC}"
    return 0
}

check_root() {
    if adb shell su -c "id" 2>/dev/null | grep -q "uid=0"; then
        echo -e "${GREEN}Root detectado!${NC}"
        return 0
    else
        echo -e "${YELLOW}Dispositivo NÃO tem root.${NC}"
        return 1
    fi
}

check_magisk() {
    if adb shell pm list packages 2>/dev/null | grep -q "com.topjohnwu.magisk"; then
        echo -e "${GREEN}Magisk detectado!${NC}"
        return 0
    else
        return 1
    fi
}

check_treble() {
    local treble=$(adb shell getprop ro.treble.enabled 2>/dev/null | tr -d '\r')
    if [ "$treble" == "true" ]; then
        echo -e "${GREEN}Project Treble: Suportado${NC}"
        return 0
    else
        echo -e "${YELLOW}Project Treble: NÃO suportado${NC}"
        return 1
    fi
}

check_bootloader() {
    local unlocked=$(adb shell getprop ro.boot.flash.locked 2>/dev/null | tr -d '\r')
    if [ "$unlocked" == "0" ]; then
        echo -e "${GREEN}Bootloader: Desbloqueado${NC}"
        return 0
    else
        echo -e "${YELLOW}Bootloader: Bloqueado${NC}"
        return 1
    fi
}

# =====================================================
# Instalação
# =====================================================

install_level_1() {
    echo -e "\n${BLUE}=== Nível 1: Debloat + BotOS Launcher ===${NC}"
    echo "Removendo bloatware e instalando BotOS como app..."
    echo ""
    
    # Executar debloat
    bash scripts/debloat.sh
}

install_level_2() {
    echo -e "\n${BLUE}=== Nível 2: Magisk Module ===${NC}"
    echo "Instalando módulo Magisk com boot animation + BotOS system app..."
    echo ""
    
    # Verificar se tem APK compilado
    if [ ! -f "../gen/android/app/build/outputs/apk/release/app-release.apk" ]; then
        echo -e "${YELLOW}APK do BotOS não encontrado.${NC}"
        echo "Compilando..."
        cd ..
        cargo tauri android build --release || {
            echo -e "${RED}Erro na compilação. Configure o ambiente Tauri primeiro.${NC}"
            return 1
        }
        cd rom
    fi
    
    # Gerar boot animation
    if [ ! -f "../bootanimation.zip" ]; then
        echo "Gerando boot animation..."
        bash ../scripts/create-bootanimation.sh || true
    fi
    
    # Build Magisk module
    bash scripts/build-magisk-module.sh
    
    # Instalar
    if [ -f "botos-magisk-v1.0.zip" ]; then
        echo "Copiando módulo para dispositivo..."
        adb push botos-magisk-v1.0.zip /sdcard/
        
        echo ""
        echo -e "${GREEN}Módulo copiado!${NC}"
        echo ""
        echo "Para completar a instalação:"
        echo "  1. Abra o Magisk Manager no celular"
        echo "  2. Vá em 'Modules' (Módulos)"
        echo "  3. Toque em '+' e selecione 'botos-magisk-v1.0.zip'"
        echo "  4. Reinicie o dispositivo"
    fi
}

install_level_3() {
    echo -e "\n${BLUE}=== Nível 3: GSI Flash ===${NC}"
    echo "Instalando GSI completa com BotOS..."
    echo ""
    
    echo -e "${YELLOW}AVISO: Isso irá APAGAR todos os dados do dispositivo!${NC}"
    echo ""
    read -p "Tem certeza? Digite 'SIM' para continuar: " confirm
    
    if [ "$confirm" != "SIM" ]; then
        echo "Cancelado."
        return 1
    fi
    
    # Verificar requisitos
    check_treble || {
        echo -e "${RED}Dispositivo não suporta Project Treble.${NC}"
        return 1
    }
    
    check_bootloader || {
        echo -e "${YELLOW}Bootloader precisa ser desbloqueado primeiro.${NC}"
        echo ""
        echo "Instruções:"
        echo "  1. Habilite 'OEM unlocking' nas opções do desenvolvedor"
        echo "  2. adb reboot bootloader"
        echo "  3. fastboot flashing unlock"
        echo "  4. Confirme no dispositivo"
        return 1
    fi
    
    echo ""
    echo "Para GSI, você precisa:"
    echo "  1. Baixar uma GSI base (ex: phh-treble)"
    echo "  2. Ou compilar AOSP com os arquivos em gsi/"
    echo ""
    echo "Veja gsi/README.md para instruções detalhadas."
}

show_device_info() {
    echo -e "\n${BLUE}=== Informações do Dispositivo ===${NC}\n"
    
    check_device || return 1
    
    echo ""
    check_root && HAS_ROOT=1 || HAS_ROOT=0
    check_magisk && HAS_MAGISK=1 || HAS_MAGISK=0
    check_treble && HAS_TREBLE=1 || HAS_TREBLE=0
    check_bootloader && HAS_UNLOCKED=1 || HAS_UNLOCKED=0
    
    echo ""
    echo -e "${CYAN}Opções disponíveis para este dispositivo:${NC}"
    echo ""
    echo -e "  ${GREEN}✓${NC} Nível 1 (Debloat + App) - Sempre disponível"
    
    if [ "$HAS_MAGISK" == "1" ]; then
        echo -e "  ${GREEN}✓${NC} Nível 2 (Magisk Module) - Magisk detectado"
    else
        echo -e "  ${YELLOW}?${NC} Nível 2 (Magisk Module) - Requer Magisk instalado"
    fi
    
    if [ "$HAS_TREBLE" == "1" ] && [ "$HAS_UNLOCKED" == "1" ]; then
        echo -e "  ${GREEN}✓${NC} Nível 3 (GSI) - Treble + Bootloader desbloqueado"
    else
        echo -e "  ${RED}✗${NC} Nível 3 (GSI) - Requer Treble + bootloader desbloqueado"
    fi
}

# =====================================================
# Menu Principal
# =====================================================

main_menu() {
    while true; do
        echo ""
        echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
        echo -e "${CYAN}                    MENU DE INSTALAÇÃO                         ${NC}"
        echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
        echo ""
        echo "  1) Ver informações do dispositivo"
        echo ""
        echo "  2) Nível 1: Debloat + BotOS Launcher (SEM root)"
        echo "     Remove bloatware via ADB, instala BotOS como app"
        echo ""
        echo "  3) Nível 2: Magisk Module (COM root)"
        echo "     Boot animation GB, BotOS como system app, bloat removido"
        echo ""
        echo "  4) Nível 3: GSI Flash (Bootloader desbloqueado)"
        echo "     Substitui Android inteiro por BotOS"
        echo ""
        echo "  5) Compilar BotOS APK"
        echo ""
        echo "  6) Gerar Boot Animation"
        echo ""
        echo "  0) Sair"
        echo ""
        read -p "Escolha [0-6]: " choice
        
        case $choice in
            1)
                check_adb && show_device_info
                ;;
            2)
                check_adb && check_device && install_level_1
                ;;
            3)
                check_adb && check_device && install_level_2
                ;;
            4)
                check_adb && check_device && install_level_3
                ;;
            5)
                echo -e "\n${BLUE}Compilando BotOS APK...${NC}\n"
                cd ..
                cargo tauri android build --release
                cd rom
                echo -e "${GREEN}APK gerado em gen/android/app/build/outputs/apk/${NC}"
                ;;
            6)
                echo -e "\n${BLUE}Gerando boot animation...${NC}\n"
                bash ../scripts/create-bootanimation.sh
                ;;
            0)
                echo -e "\n${GREEN}Até logo!${NC}\n"
                exit 0
                ;;
            *)
                echo -e "${RED}Opção inválida${NC}"
                ;;
        esac
        
        echo ""
        read -p "Pressione ENTER para continuar..."
        clear
    done
}

# =====================================================
# Início
# =====================================================

# Verificar se ADB está disponível
check_adb || exit 1

# Mostrar menu
main_menu
