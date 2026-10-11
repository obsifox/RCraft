# Minecraft 1.12.2 (Protocol 340) Specification Details

## State Flow
1. **Handshake (Serverbound 0x00)**
   - Protocol Version: 340
   - Server Address: String
   - Server Port: Unsigned Short
   - Next State: 1 (Status) or 2 (Login)

2. **Login State**
   - Client -> Server: `Login Start (0x00)` with `username`
   - Server -> Client: `Set Compression (0x03)` (Optional, enables zlib if size >= threshold)
   - Server -> Client: `Login Success (0x02)` with `UUID` string and `username`

3. **Play State**
   - Server -> Client: `Join Game (0x23)`
   - Server -> Client: `Player Position And Look (0x2F)`
   - Client -> Server: `Teleport Confirm (0x00)`
   - Client -> Server: `Client Settings (0x04)`
   - Server -> Client: `Chunk Data (0x20)` (16x16x256 voxel column)
   - Server <-> Client: `Keep Alive` (Server 0x1F, Client 0x0B)
   - Server <-> Client: `Chat Message` (Client 0x02, Server 0x0F)
