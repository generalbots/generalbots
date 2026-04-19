#!/bin/bash
set -e

echo "Cleaning up..."
rm -rf botserver-stack/ ./work/ .env

echo "Starting services..."
./restart.sh

echo "Reset complete!"
