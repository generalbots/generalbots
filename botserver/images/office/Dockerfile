# #2062 — pre-fabricated office workspace image (Debian 13 + headless Chromium).
#
# Build:
#   docker build -f botserver/images/office/Dockerfile -t gb-workspace:office .
# Import into Incus:
#   docker save gb-workspace:office | incus image import - --alias gb-workspace-office
#
# Profile contract: `office` adds headless Chromium (print-to-PDF and
# screenshots), fonts and the document libraries on top of the base runtime.
# It deliberately ships no LibreOffice, no ffmpeg and no torch/tensorflow/CUDA:
# audio/video/AI work runs in BotModels, and the `media`/`data` profiles are
# opt-in (`GB_VM_PROFILE`). The size guard is check-size.sh.

FROM debian:13-slim AS base

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -q \
 && apt-get install -y -q --no-install-recommends \
      python3 python3-pip python3-venv nodejs npm git ca-certificates iptables iproute2 cron curl \
 && rm -rf /var/lib/apt/lists/*

FROM base AS office

# Debian's `chromium` ships the headless mode used by --headless=new.
RUN apt-get update -q \
 && apt-get install -y -q --no-install-recommends \
      chromium fonts-noto-core fonts-dejavu-core \
 && rm -rf /var/lib/apt/lists/*

# Document generation is library-first; Chromium only renders HTML to PDF.
RUN python3 -m pip install --break-system-packages --no-cache-dir --quiet \
      pandas numpy openpyxl python-pptx pillow matplotlib requests python-docx \
      reportlab beautifulsoup4 lxml tabulate pikepdf pypdf markdown jinja2

RUN mkdir -p /opt/vibe/conversations \
 && id -u runner >/dev/null 2>&1 || useradd -m -s /bin/bash runner \
 && chown -R runner:runner /opt/vibe \
 && echo "v6-office" > /opt/vibe/.provisioned

COPY botserver/images/office/check-size.sh /usr/local/sbin/check-vm-size.sh
RUN chmod +x /usr/local/sbin/check-vm-size.sh

CMD ["/bin/bash"]
