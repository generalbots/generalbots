#!/bin/bash
#
# fix-collabora-repo.sh
# Removes broken Collabora Online repositories that may cause apt update failures
#

if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

echo "Removing Collabora repository configurations..."

# Remove specific lists if they exist
rm -f /etc/apt/sources.list.d/collabora.list
rm -f /etc/apt/sources.list.d/collaboraoffice.list

# Remove entries from other files
grep -r "collaboraoffice" /etc/apt/sources.list.d/ | cut -d: -f1 | sort | uniq | while read -r file; do
    echo "Cleaning $file..."
    sed -i '/collaboraoffice/d' "$file"
done

sed -i '/collaboraoffice/d' /etc/apt/sources.list

echo "Updating apt cache..."
apt-get update

echo "Done."
