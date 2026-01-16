import json
import os

print("🔄 Merging cargo-sources.json into org.carmenta.App.json...")

manifest = {
    "app-id": "org.carmenta.App",
    "runtime": "org.gnome.Platform",
    "runtime-version": "47",
    "sdk": "org.gnome.Sdk",
    "command": "carmenta",
    "finish-args": [
        "--share=ipc",
        "--socket=fallback-x11",
        "--socket=wayland",
        "--device=dri",
        "--talk-name=org.gnome.Shell"
    ],
    "build-options": {
        "env": {
             "CARGO_HOME": "/run/build/carmenta/cargo"
        }
    },
    "modules": [
        {
            "name": "carmenta",
            "buildsystem": "simple",
            "build-commands": [
                "cargo build --release --offline",
                "install -D target/release/carmenta /app/bin/carmenta",
                "install -D data/org.carmenta.App.desktop /app/share/applications/org.carmenta.App.desktop",
                "install -D data/org.carmenta.App.svg /app/share/icons/hicolor/scalable/apps/org.carmenta.App.svg"
            ],
            "sources": [
                { "type": "dir", "path": "." }
            ]
        }
    ]
}

try:
    with open("cargo-sources.json", "r") as f:
        cargo_sources = json.load(f)
    
    # append cargo sources
    manifest["modules"][0]["sources"].extend(cargo_sources)

    with open("org.carmenta.App.json", "w") as f:
        json.dump(manifest, f, indent=4)
        
    print("✅ Created org.carmenta.App.json (Full Manifest)")

except Exception as e:
    print(f"❌ Error: {e}")
    exit(1)
