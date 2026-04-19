# BotOS Device Configuration

# Inherit from common
$(call inherit-product, $(SRC_TARGET_DIR)/product/core_64_bit.mk)
$(call inherit-product, $(SRC_TARGET_DIR)/product/full_base.mk)

# Product info
PRODUCT_NAME := botos
PRODUCT_DEVICE := botos
PRODUCT_BRAND := Pragmatismo
PRODUCT_MODEL := BotOS
PRODUCT_MANUFACTURER := Pragmatismo.io

# Locales
PRODUCT_LOCALES := pt_BR en_US es_ES

# =====================================================
# BotOS Customizations
# =====================================================

# BotOS Launcher como app de sistema privilegiado
PRODUCT_PACKAGES += \
    BotOS

# Boot animation customizada
PRODUCT_COPY_FILES += \
    device/pragmatismo/botos/media/bootanimation.zip:system/media/bootanimation.zip

# Overlay para configurações padrão
DEVICE_PACKAGE_OVERLAYS += device/pragmatismo/botos/overlay

# Propriedades do sistema
PRODUCT_PROPERTY_OVERRIDES += \
    ro.product.brand=Pragmatismo \
    ro.product.name=BotOS \
    ro.product.device=botos \
    ro.build.display.id=BotOS-1.0 \
    ro.botos.version=1.0 \
    persist.sys.language=pt \
    persist.sys.country=BR

# Desabilitar analytics/telemetria
PRODUCT_PROPERTY_OVERRIDES += \
    ro.com.google.gmsversion= \
    ro.setupwizard.mode=DISABLED

# Performance
PRODUCT_PROPERTY_OVERRIDES += \
    dalvik.vm.heapsize=512m \
    dalvik.vm.heapgrowthlimit=256m

# =====================================================
# Removido bloatware
# =====================================================

# NÃO incluir esses pacotes
PRODUCT_PACKAGES_REMOVED += \
    Calculator \
    Calendar \
    Camera2 \
    DeskClock \
    Email \
    Gallery2 \
    Music \
    QuickSearchBox

# =====================================================
# Apps mínimos necessários
# =====================================================

PRODUCT_PACKAGES += \
    Launcher3 \
    Settings \
    SystemUI \
    SettingsProvider \
    Shell \
    adb
