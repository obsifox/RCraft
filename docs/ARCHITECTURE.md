# rMinecraft - Native Rust Re-implementation of Minecraft 1.12.2 (Protocol 340)

## 1. Project Overview & Commercial Strategy
rMinecraft is a ground-up recreation of the Minecraft Java Edition 1.12.2 client and protocol stack in Rust.
The purpose of this project is to deliver a zero-GC, ultra-high-performance native binary replacement for the legacy Java client, enabling existing Java players who purchased the game to seamlessly connect to vanilla and modded Minecraft 1.12.2 servers with 10x-50x better memory efficiency, instant startup, zero garbage-collection stutter, and modern graphics/textures.

## 2. Workspace & Crate Structure
- **crates/rminecraft-protocol**:
  - Implementation of Minecraft Protocol 340 (Release 1.12.2).
  - High-performance VarInt / VarLong codecs.
  - Zlib packet compression/decompression handling.
  - State machine: Handshaking -> Status -> Login -> Play.
  - Mojang offline UUID derivation (`offline_player_uuid` using MD5 and version 3 / variant 2).
- **crates/rminecraft-world**:
  - Voxel world representation with 16x16x16 ChunkSections and 16x256x16 ChunkColumns.
  - Fast bit-unpacking of 1.12.2 chunk data formats.
  - 3D Voxel Mesher with hidden-face culling (Top, Bottom, North, South, East, West).
  - Block registry & texture coordinate mappings.
- **crates/rminecraft-client**:
  - Native client executable accepting standard Mojang launcher arguments.
  - Multi-threaded network stream and packet dispatcher.
  - Real-time chunk meshing and world storage.
  - Player physics simulation: gravity (-0.08 / tick), drag, ground collision detection.
  - Keep-alive heartbeat and teleport confirmation.
- **crates/rminecraft-launcher**:
  - Dedicated offline account manager.
  - Custom skin and cape injection (`.minecraft/skins/`, `.minecraft/capes/`).
  - Strict compliance with official `.minecraft` directory structure (`versions/`, `assets/`, `launcher_profiles.json`).
- **crates/rminecraft-server**:
  - Test server implementation in Rust for offline development and protocol validation.
- **tools/texture_migrator.py**:
  - Automated downloader for 1.12.2 and latest modern release (26.x).
  - Texture catalog extractor and cross-version renaming mapper.
  - Generates ready-to-use modernized resource packs.
