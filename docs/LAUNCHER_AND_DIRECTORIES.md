# rMinecraft Official Directory Architecture

The launcher and client use the exact directory layout as official Minecraft:
```
.minecraft/
├── launcher_profiles.json      # Stores local accounts, selected profile, authDatabase
├── versions/
│   └── 1.12.2-rust/
│       ├── 1.12.2-rust.json    # Standard Mojang version manifest
│       └── rminecraft-client   # Native Rust client binary
├── assets/
│   ├── indexes/
│   │   └── 1.12.json
│   └── objects/
├── skins/                      # Local offline custom skins (.png)
├── capes/                      # Local offline custom capes (.png)
├── resourcepacks/              # Modernized textures pack (modern_textures_1.12.2.zip)
├── saves/                      # World saves
└── logs/                       # Client and launcher logs
```
