#!/bin/bash
set -e

echo "Cleaning up..."
rm -rf botserver-stack/ botserver/botserver-stack/ ./work/ botserver/work/ .env botserver/.env \
  botserver.log botserver/botserver.log botui.log botserver/botui.log botmodels.log botserver/botmodels.log

echo "Starting services..."
./restart.sh

echo "Reset complete!"
