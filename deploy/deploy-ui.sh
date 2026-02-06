#!/bin/bash

set -e

DEPLOY_DIR="${1:-/opt/gbo}"
SRC_DIR="$(dirname "$0")/../.."

echo "Deploying UI files to $DEPLOY_DIR"

mkdir -p "$DEPLOY_DIR/bin/ui/suite"

cp -r "$SRC_DIR/botui/ui/suite/"* "$DEPLOY_DIR/bin/ui/suite/"

echo "UI files deployed successfully"
echo "Location: $DEPLOY_DIR/bin/ui/suite"
ls -la "$DEPLOY_DIR/bin/ui/suite" | head -20