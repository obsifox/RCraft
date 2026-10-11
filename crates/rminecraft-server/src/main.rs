use std::io::{self, Cursor, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use byteorder::{BigEndian, WriteBytesExt};
use rminecraft_protocol::*;
use rminecraft_world::*;

fn main() -> io::Result<()> {
    let port = 25565;
    let bind_addr = format!("0.0.0.0:{}", port);
    println!("[rMinecraft Server] Starting 1.12.2 Protocol 340 Server on {}", bind_addr);
    let listener = TcpListener::bind(&bind_addr)?;
    println!("[rMinecraft Server] Listening for connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || {
                    if let Err(e) = handle_client(s) {
                        eprintln!("[Server] Connection error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("[Server] Accept error: {}", e),
        }
    }
    Ok(())
}

fn handle_client(stream: TcpStream) -> io::Result<()> {
    let peer = stream.peer_addr()?;
    println!("[Server] New connection from {}", peer);
    let mut packet_stream = PacketStream::new(stream)?;

    // 1. Handshake
    let raw = packet_stream.read_packet()?;
    let mut cursor = Cursor::new(raw.payload);
    let handshake = HandshakePacket::read(&mut cursor)?;
    println!("[Server] Handshake received: proto={}, addr={}:{}, next_state={}",
             handshake.protocol_version, handshake.server_address, handshake.server_port, handshake.next_state);

    if handshake.next_state == 1 {
        // Status State
        handle_status(&mut packet_stream)?;
    } else if handshake.next_state == 2 {
        // Login State
        handle_login(&mut packet_stream)?;
    }
    Ok(())
}

fn handle_status(stream: &mut PacketStream) -> io::Result<()> {
    loop {
        let raw = stream.read_packet()?;
        if raw.id == 0x00 {
            // Status Request
            let status = serde_json::json!({
                "version": {
                    "name": "1.12.2 (rMinecraft-Rust)",
                    "protocol": PROTOCOL_VERSION_1_12_2
                },
                "players": {
                    "max": 20,
                    "online": 1,
                    "sample": [
                        {"name": "RustPlayer", "id": "4566e69f-c907-48ee-8d71-d7ba5aa00d20"}
                    ]
                },
                "description": {
                    "text": "§a§lrMinecraft §7Native Rust 1.12.2 Server §e[Test Prototype]"
                }
            });
            let json_str = status.to_string();
            let mut payload = Vec::new();
            write_string(&mut payload, &json_str)?;
            stream.write_packet(0x00, &payload)?;
            println!("[Server] Responded with MOTD and status JSON");
        } else if raw.id == 0x01 {
            // Ping packet
            stream.write_packet(0x01, &raw.payload)?;
            println!("[Server] Pong sent, closing status connection");
            break;
        }
    }
    Ok(())
}

fn handle_login(stream: &mut PacketStream) -> io::Result<()> {
    let raw = stream.read_packet()?;
    if raw.id != 0x00 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected LoginStart packet"));
    }
    let mut cursor = Cursor::new(raw.payload);
    let login_start = LoginStartPacket::read(&mut cursor)?;
    println!("[Server] Player logging in: '{}'", login_start.username);

    // Compute offline UUID
    let uuid = offline_player_uuid(&login_start.username);
    let uuid_str = uuid.hyphenated().to_string();
    println!("[Server] Assigning offline UUID: {}", uuid_str);

    // Send Login Success (Packet 0x02)
    let login_success = LoginSuccessPacket {
        uuid: uuid_str,
        username: login_start.username.clone(),
    };
    let mut success_payload = Vec::new();
    login_success.write(&mut success_payload)?;
    stream.write_packet(0x02, &success_payload)?;
    println!("[Server] Login success packet sent!");

    // Transition to Play State
    handle_play(stream, login_start.username)?;
    Ok(())
}

fn handle_play(stream: &mut PacketStream, username: String) -> io::Result<()> {
    println!("[Server] Entering Play state for player: {}", username);

    // 1. Join Game (0x23 in 1.12.2)
    let join_game = JoinGamePacket {
        entity_id: 1001,
        gamemode: 1, // Creative for testing
        dimension: 0, // Overworld
        difficulty: 1, // Easy
        max_players: 50,
        level_type: "default".to_string(),
        reduced_debug_info: false,
    };
    let mut join_payload = Vec::new();
    join_game.write(&mut join_payload)?;
    stream.write_packet(0x23, &join_payload)?;
    println!("[Server] Sent Join Game packet (Entity ID 1001)");

    // 2. Spawn Position (0x46 in 1.12.2)
    let spawn_pos = Position::new(0, 64, 0);
    let mut spawn_payload = Vec::new();
    spawn_pos.write(&mut spawn_payload)?;
    stream.write_packet(0x46, &spawn_payload)?;

    // 3. Player Position & Look (0x2F in 1.12.2)
    let pos_and_look = PlayerPositionAndLookPacket {
        x: 0.5,
        y: 65.0,
        z: 0.5,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0,
        teleport_id: 1,
    };
    let mut pos_payload = Vec::new();
    pos_and_look.write(&mut pos_payload)?;
    stream.write_packet(0x2F, &pos_payload)?;
    println!("[Server] Sent initial player position (0.5, 65.0, 0.5)");

    // 4. Send 3x3 Chunks around spawn
    send_initial_chunks(stream)?;

    // 5. Welcome Chat Message (0x0F in 1.12.2)
    let chat_json = serde_json::json!({
        "text": format!("Welcome §a{}§f to the §6rMinecraft §7(Rust 1.12.2) server!", username)
    }).to_string();
    let chat_pkt = ChatMessagePacket {
        message: chat_json,
        position: 0,
    };
    let mut chat_payload = Vec::new();
    chat_pkt.write(&mut chat_payload)?;
    stream.write_packet(0x0F, &chat_payload)?;

    // Start KeepAlive thread
    let mut write_stream = stream.try_clone()?;
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();

    thread::spawn(move || {
        let mut keep_alive_id = 1000i64;
        while r_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(5));
            let ka = KeepAlivePacket { id: keep_alive_id };
            let mut buf = Vec::new();
            if ka.write(&mut buf).is_ok() {
                if write_stream.write_packet(0x1F, &buf).is_err() {
                    break;
                }
            }
            keep_alive_id += 1;
        }
    });

    // Main packet loop
    loop {
        let raw = match stream.read_packet() {
            Ok(pkt) => pkt,
            Err(e) => {
                println!("[Server] Player disconnected: {}", e);
                running.store(false, Ordering::Relaxed);
                break;
            }
        };

        match raw.id {
            0x00 => {
                // Teleport Confirm
                let mut c = Cursor::new(raw.payload);
                let tele_id = read_varint(&mut c).unwrap_or(0);
                println!("[Server] Player confirmed teleport ID: {}", tele_id);
            }
            0x02 => {
                // Serverbound Chat
                let mut c = Cursor::new(raw.payload);
                if let Ok(msg) = read_string(&mut c) {
                    println!("[Server Chat] <{}> {}", username, msg);
                    // Echo chat back
                    let broadcast_json = serde_json::json!({
                        "text": format!("<{}> {}", username, msg)
                    }).to_string();
                    let echo = ChatMessagePacket {
                        message: broadcast_json,
                        position: 0,
                    };
                    let mut b = Vec::new();
                    echo.write(&mut b)?;
                    stream.write_packet(0x0F, &b)?;
                }
            }
            0x0B => {
                // Keep Alive Response
                // Client alive
            }
            0x0D | 0x0E => {
                // Player Position / Position & Look
                // Client moved
            }
            _ => {
                // Other packets
            }
        }
    }
    Ok(())
}

fn send_initial_chunks(stream: &mut PacketStream) -> io::Result<()> {
    // Generate a 3x3 grid of chunks centered at (0,0)
    for cx in -1..=1 {
        for cz in -1..=1 {
            let chunk_data = build_flat_chunk_data();
            let mut pkt_data = Vec::new();
            pkt_data.write_i32::<BigEndian>(cx)?;
            pkt_data.write_i32::<BigEndian>(cz)?;
            pkt_data.write_u8(1)?; // full chunk
            write_varint(&mut pkt_data, 0x0F)?; // Bit mask: lower 4 sections (Y: 0..64)
            write_varint(&mut pkt_data, chunk_data.len() as i32)?;
            pkt_data.extend_from_slice(&chunk_data);
            write_varint(&mut pkt_data, 0)?; // 0 block entities
            stream.write_packet(0x20, &pkt_data)?;
        }
    }
    println!("[Server] Sent 3x3 chunks around spawn (X:-1..1, Z:-1..1)");
    Ok(())
}

fn build_flat_chunk_data() -> Vec<u8> {
    let mut data = Vec::new();

    // 4 sections (Y 0..64):
    // Section 0 (Y 0..16): Bedrock (Y=0), Stone (Y=1..15)
    // Section 1 (Y 16..32): Stone
    // Section 2 (Y 32..48): Stone
    // Section 3 (Y 48..64): Dirt (Y=48..62), Grass (Y=63)
    for s in 0..4 {
        // Bits per block: 13 (unpaletted)
        data.push(13);
        // Data array: 4096 blocks * 13 bits = 53248 bits = 832 u64s
        let u64_count = 832;
        write_varint(&mut data, u64_count as i32).unwrap();

        let mut long_vals = vec![0u64; u64_count];
        let mut bit_offset = 0;

        for y in 0..16 {
            let actual_y = s * 16 + y;
            for z in 0..16 {
                for x in 0..16 {
                    let block_id = if actual_y == 0 {
                        BlockState::BEDROCK.to_raw()
                    } else if actual_y < 60 {
                        BlockState::STONE.to_raw()
                    } else if actual_y < 63 {
                        BlockState::DIRT.to_raw()
                    } else if actual_y == 63 {
                        BlockState::GRASS.to_raw()
                    } else {
                        BlockState::AIR.to_raw()
                    };

                    let u64_idx = bit_offset / 64;
                    let bit_in_u64 = bit_offset % 64;

                    long_vals[u64_idx] |= (block_id as u64 & 0x1FFF) << bit_in_u64;
                    if bit_in_u64 + 13 > 64 && u64_idx + 1 < u64_count {
                        let spill_bits = (bit_in_u64 + 13) - 64;
                        long_vals[u64_idx + 1] |= (block_id as u64 & 0x1FFF) >> (13 - spill_bits);
                    }
                    bit_offset += 13;
                }
            }
        }

        for val in long_vals {
            data.write_u64::<BigEndian>(val).unwrap();
        }

        // Block light: 2048 bytes
        data.extend(vec![0u8; 2048]);
        // Sky light: 2048 bytes (fully lit: 0xFF)
        data.extend(vec![0xFFu8; 2048]);
    }

    // Biomes: 256 bytes (Plains = 1)
    data.extend(vec![1u8; 256]);
    data
}
