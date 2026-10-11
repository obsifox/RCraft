use std::io::{self, Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use md5::{Md5, Digest};
use uuid::Uuid;

pub const PROTOCOL_VERSION_1_12_2: i32 = 340;

pub fn read_varint<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut num_read = 0;
    let mut result = 0i32;
    loop {
        let read = reader.read_u8()?;
        let value = (read & 0x7F) as i32;
        result |= value << (7 * num_read);
        num_read += 1;
        if num_read > 5 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt is too big"));
        }
        if (read & 0x80) == 0 {
            break;
        }
    }
    Ok(result)
}

pub fn write_varint<W: Write>(writer: &mut W, mut value: i32) -> io::Result<()> {
    loop {
        let mut temp = (value & 0x7F) as u8;
        // In Rust, right shift of signed integer is arithmetic, cast to unsigned for logical shift
        let uval = value as u32;
        if (uval >> 7) != 0 {
            temp |= 0x80;
            writer.write_u8(temp)?;
            value = (uval >> 7) as i32;
        } else {
            writer.write_u8(temp)?;
            break;
        }
    }
    Ok(())
}

pub fn varint_size(value: i32) -> usize {
    let mut v = value as u32;
    let mut count = 0;
    loop {
        count += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    count
}

pub fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = read_varint(reader)? as usize;
    if len > 32767 * 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "String length exceeds limit"));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let bytes = value.as_bytes();
    write_varint(writer, bytes.len() as i32)?;
    writer.write_all(bytes)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let val = reader.read_u64::<BigEndian>()?;
        let mut x = (val >> 38) as i32;
        let mut y = ((val >> 26) & 0xFFF) as i32;
        let mut z = (val & 0x3FFFFFF) as i32;

        if x >= 1 << 25 { x -= 1 << 26; }
        if y >= 1 << 11 { y -= 1 << 12; }
        if z >= 1 << 25 { z -= 1 << 26; }

        Ok(Position { x, y, z })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let x = (self.x as i64) & 0x3FFFFFF;
        let y = (self.y as i64) & 0xFFF;
        let z = (self.z as i64) & 0x3FFFFFF;
        let encoded = (x << 38) | (y << 26) | z;
        writer.write_u64::<BigEndian>(encoded as u64)
    }
}

/// Computes the exact Minecraft Java offline-player UUID from username
/// Java equivalent: UUID.nameUUIDFromBytes(("OfflinePlayer:" + username).getBytes(StandardCharsets.UTF_8))
pub fn offline_player_uuid(username: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", username).as_bytes());
    let mut bytes = hasher.finalize();

    // Set UUID version to 3 (name-based MD5)
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    // Set UUID variant to IETF variant (variant 2)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes.into())
}
