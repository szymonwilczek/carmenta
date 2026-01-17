#!/bin/bash
set -e

echo "📦 Packaging GNOME Extension..."
ZIP_NAME="carmenta-extension.zip"
rm -f "$ZIP_NAME"

cd extension
zip -r ../"$ZIP_NAME" extension.js metadata.json schemas/org.gnome.shell.extensions.carmenta.gschema.xml

echo "✅ Created $ZIP_NAME"
