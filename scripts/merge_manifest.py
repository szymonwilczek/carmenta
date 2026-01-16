import json
import os

print("🔄 Merging cargo-sources.json into io.github.szymonwilczek.carmenta.json...")

manifest = {
    "app-id": "io.github.szymonwilczek.carmenta",
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
                "install -D data/io.github.szymonwilczek.carmenta.desktop /app/share/applications/io.github.szymonwilczek.carmenta.desktop",
                "install -D data/io.github.szymonwilczek.carmenta.svg /app/share/icons/hicolor/scalable/apps/io.github.szymonwilczek.carmenta.svg",
                "install -D data/io.github.szymonwilczek.carmenta.metainfo.xml /app/share/metainfo/io.github.szymonwilczek.carmenta.metainfo.xml"
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

    with open("io.github.szymonwilczek.carmenta.json", "w") as f:
        json.dump(manifest, f, indent=4)
        
    print("✅ Created io.github.szymonwilczek.carmenta.json (Full Manifest)")

except Exception as e:
    print(f"❌ Error: {e}")
    exit(1)
