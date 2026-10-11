use std::io::{self, Cursor, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play,
}

// -------------------------------------------------------------
// Handshake Packets
// -------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32, // 1 for Status, 2 for Login
}

impl HandshakePacket {
    pub fn new(host: &str, port: u16, next_state: i32) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION_1_12_2,
            server_address: host.to_string(),
            server_port: port,
            next_state,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let protocol_version = read_varint(reader)?;
        let server_address = read_string(reader)?;
        let server_port = reader.read_u16::<BigEndian>()?;
        let next_state = read_varint(reader)?;
        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_varint(writer, self.protocol_version)?;
        write_string(writer, &self.server_address)?;
        writer.write_u16::<BigEndian>(self.server_port)?;
        write_varint(writer, self.next_state)?;
        Ok(())
    }
}

// -------------------------------------------------------------
// Status Packets
// -------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub version: Option<VersionInfo>,
    pub players: Option<PlayersInfo>,
    pub description: Option<serde_json::Value>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayersInfo {
    pub max: i32,
    pub online: i32,
    pub sample: Option<Vec<PlayerSample>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

// -------------------------------------------------------------
// Login Packets
// -------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct LoginStartPacket {
    pub username: String,
}

impl LoginStartPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.username)
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let username = read_string(reader)?;
        Ok(Self { username })
    }
}

#[derive(Debug, Clone)]
pub struct LoginSuccessPacket {
    pub uuid: String,
    pub username: String,
}

impl LoginSuccessPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let uuid = read_string(reader)?;
        let username = read_string(reader)?;
        Ok(Self { uuid, username })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.uuid)?;
        write_string(writer, &self.username)?;
        Ok(())
    }
}

// -------------------------------------------------------------
// Play Packets (Clientbound)
// -------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct JoinGamePacket {
    pub entity_id: i32,
    pub gamemode: u8,
    pub dimension: i32,
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}

impl JoinGamePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let entity_id = reader.read_i32::<BigEndian>()?;
        let gamemode = reader.read_u8()?;
        let dimension = reader.read_i32::<BigEndian>()?;
        let difficulty = reader.read_u8()?;
        let max_players = reader.read_u8()?;
        let level_type = read_string(reader)?;
        let reduced_debug_info = reader.read_u8()? != 0;
        Ok(Self {
            entity_id,
            gamemode,
            dimension,
            difficulty,
            max_players,
            level_type,
            reduced_debug_info,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i32::<BigEndian>(self.entity_id)?;
        writer.write_u8(self.gamemode)?;
        writer.write_i32::<BigEndian>(self.dimension)?;
        writer.write_u8(self.difficulty)?;
        writer.write_u8(self.max_players)?;
        write_string(writer, &self.level_type)?;
        writer.write_u8(if self.reduced_debug_info { 1 } else { 0 })?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerPositionAndLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: i32,
}

impl PlayerPositionAndLookPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let x = reader.read_f64::<BigEndian>()?;
        let y = reader.read_f64::<BigEndian>()?;
        let z = reader.read_f64::<BigEndian>()?;
        let yaw = reader.read_f32::<BigEndian>()?;
        let pitch = reader.read_f32::<BigEndian>()?;
        let flags = reader.read_u8()?;
        let teleport_id = read_varint(reader)?;
        Ok(Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
            teleport_id,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_f64::<BigEndian>(self.x)?;
        writer.write_f64::<BigEndian>(self.y)?;
        writer.write_f64::<BigEndian>(self.z)?;
        writer.write_f32::<BigEndian>(self.yaw)?;
        writer.write_f32::<BigEndian>(self.pitch)?;
        writer.write_u8(self.flags)?;
        write_varint(writer, self.teleport_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChunkDataPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub full_chunk: bool,
    pub primary_bit_mask: i32,
    pub data: Vec<u8>,
}

impl ChunkDataPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let chunk_x = reader.read_i32::<BigEndian>()?;
        let chunk_z = reader.read_i32::<BigEndian>()?;
        let full_chunk = reader.read_u8()? != 0;
        let primary_bit_mask = read_varint(reader)?;
        let size = read_varint(reader)? as usize;
        let mut data = vec![0u8; size];
        reader.read_exact(&mut data)?;
        // After data is number of block entities, but in 1.12.2 read remaining if any
        Ok(Self {
            chunk_x,
            chunk_z,
            full_chunk,
            primary_bit_mask,
            data,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i32::<BigEndian>(self.chunk_x)?;
        writer.write_i32::<BigEndian>(self.chunk_z)?;
        writer.write_u8(if self.full_chunk { 1 } else { 0 })?;
        write_varint(writer, self.primary_bit_mask)?;
        write_varint(writer, self.data.len() as i32)?;
        writer.write_all(&self.data)?;
        write_varint(writer, 0)?; // 0 block entities
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessagePacket {
    pub message: String,
    pub position: u8, // 0: chat box, 1: system message, 2: action bar
}

impl ChatMessagePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let message = read_string(reader)?;
        let position = reader.read_u8().unwrap_or(0);
        Ok(Self { message, position })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.message)?;
        writer.write_u8(self.position)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct KeepAlivePacket {
    pub id: i64,
}

impl KeepAlivePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let id = reader.read_i64::<BigEndian>()?;
        Ok(Self { id })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i64::<BigEndian>(self.id)
    }
}

// -------------------------------------------------------------
// Play Packets (Serverbound)
// -------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct TeleportConfirmPacket {
    pub teleport_id: i32,
}

impl TeleportConfirmPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_varint(writer, self.teleport_id)
    }
}

#[derive(Debug, Clone)]
pub struct ServerboundChatPacket {
    pub message: String,
}

impl ServerboundChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.message)
    }
}

#[derive(Debug, Clone)]
pub struct ClientSettingsPacket {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
}

impl Default for ClientSettingsPacket {
    fn default() -> Self {
        Self {
            locale: "en_US".to_string(),
            view_distance: 10,
            chat_mode: 0,
            chat_colors: true,
            displayed_skin_parts: 0x7F, // all parts visible (jacket, sleeves, pants, hat)
            main_hand: 1, // right hand
        }
    }
}

impl ClientSettingsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.locale)?;
        writer.write_i8(self.view_distance)?;
        write_varint(writer, self.chat_mode)?;
        writer.write_u8(if self.chat_colors { 1 } else { 0 })?;
        writer.write_u8(self.displayed_skin_parts)?;
        write_varint(writer, self.main_hand)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerPositionAndLookServerbound {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl PlayerPositionAndLookServerbound {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_f64::<BigEndian>(self.x)?;
        writer.write_f64::<BigEndian>(self.y)?;
        writer.write_f64::<BigEndian>(self.z)?;
        writer.write_f32::<BigEndian>(self.yaw)?;
        writer.write_f32::<BigEndian>(self.pitch)?;
        writer.write_u8(if self.on_ground { 1 } else { 0 })?;
        Ok(())
    }
}

// -------------------------------------------------------------
// Extended Play Packets (Protocol 340 / 1.12.2)
// -------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BlockChangePacket {
    pub position: Position,
    pub block_id: i32,
}

impl BlockChangePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let position = Position::read(reader)?;
        let block_id = read_varint(reader)?;
        Ok(Self { position, block_id })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.position.write(writer)?;
        write_varint(writer, self.block_id)
    }
}

#[derive(Debug, Clone)]
pub struct TimeUpdatePacket {
    pub world_age: i64,
    pub time_of_day: i64,
}

impl TimeUpdatePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let world_age = reader.read_i64::<BigEndian>()?;
        let time_of_day = reader.read_i64::<BigEndian>()?;
        Ok(Self { world_age, time_of_day })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_i64::<BigEndian>(self.world_age)?;
        writer.write_i64::<BigEndian>(self.time_of_day)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdateHealthPacket {
    pub health: f32,
    pub food: i32,
    pub food_saturation: f32,
}

impl UpdateHealthPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let health = reader.read_f32::<BigEndian>()?;
        let food = read_varint(reader)?;
        let food_saturation = reader.read_f32::<BigEndian>()?;
        Ok(Self { health, food, food_saturation })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_f32::<BigEndian>(self.health)?;
        write_varint(writer, self.food)?;
        writer.write_f32::<BigEndian>(self.food_saturation)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerDiggingPacket {
    pub status: i32, // 0: Started digging, 1: Cancelled digging, 2: Finished digging, 3: Drop item stack, 4: Drop item, 5: Shoot arrow / finish eating, 6: Swap item in hand
    pub position: Position,
    pub face: i8, // 0: -Y, 1: +Y, 2: -Z, 3: +Z, 4: -X, 5: +X
}

impl PlayerDiggingPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_varint(writer, self.status)?;
        self.position.write(writer)?;
        writer.write_i8(self.face)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerBlockPlacementPacket {
    pub position: Position,
    pub face: i32, // 0..5
    pub hand: i32, // 0: main hand, 1: off hand
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_z: f32,
}

impl PlayerBlockPlacementPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.position.write(writer)?;
        write_varint(writer, self.face)?;
        write_varint(writer, self.hand)?;
        writer.write_f32::<BigEndian>(self.cursor_x)?;
        writer.write_f32::<BigEndian>(self.cursor_y)?;
        writer.write_f32::<BigEndian>(self.cursor_z)?;
        Ok(())
    }
}
