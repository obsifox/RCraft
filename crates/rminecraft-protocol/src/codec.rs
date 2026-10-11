use std::io::{self, Cursor, Read, Write};
use std::net::TcpStream;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use crate::types::{read_varint, write_varint};

pub struct PacketStream {
    stream: TcpStream,
    compression_threshold: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub id: i32,
    pub payload: Vec<u8>,
}

impl PacketStream {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            compression_threshold: None,
        })
    }

    pub fn set_compression(&mut self, threshold: i32) {
        if threshold >= 0 {
            self.compression_threshold = Some(threshold);
        } else {
            self.compression_threshold = None;
        }
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            stream: self.stream.try_clone()?,
            compression_threshold: self.compression_threshold,
        })
    }

    pub fn read_packet(&mut self) -> io::Result<RawPacket> {
        let packet_length = read_varint(&mut self.stream)?;
        if packet_length <= 0 || packet_length > 2097151 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid packet length: {}", packet_length)));
        }

        let mut packet_buf = vec![0u8; packet_length as usize];
        self.stream.read_exact(&mut packet_buf)?;
        let mut cursor = Cursor::new(packet_buf);

        if let Some(_threshold) = self.compression_threshold {
            let data_length = read_varint(&mut cursor)?;
            if data_length == 0 {
                // Uncompressed data
                let packet_id = read_varint(&mut cursor)?;
                let pos = cursor.position() as usize;
                let payload = cursor.into_inner()[pos..].to_vec();
                Ok(RawPacket { id: packet_id, payload })
            } else {
                // Compressed data with zlib
                let pos = cursor.position() as usize;
                let compressed_slice = &cursor.into_inner()[pos..];
                let mut decoder = ZlibDecoder::new(compressed_slice);
                let mut decompressed = Vec::with_capacity(data_length as usize);
                decoder.read_to_end(&mut decompressed)?;

                let mut dec_cursor = Cursor::new(decompressed);
                let packet_id = read_varint(&mut dec_cursor)?;
                let dec_pos = dec_cursor.position() as usize;
                let payload = dec_cursor.into_inner()[dec_pos..].to_vec();
                Ok(RawPacket { id: packet_id, payload })
            }
        } else {
            let packet_id = read_varint(&mut cursor)?;
            let pos = cursor.position() as usize;
            let payload = cursor.into_inner()[pos..].to_vec();
            Ok(RawPacket { id: packet_id, payload })
        }
    }

    pub fn write_packet(&mut self, packet_id: i32, payload: &[u8]) -> io::Result<()> {
        if let Some(threshold) = self.compression_threshold {
            let mut uncompressed_buf = Vec::new();
            write_varint(&mut uncompressed_buf, packet_id)?;
            uncompressed_buf.extend_from_slice(payload);

            let uncompressed_len = uncompressed_buf.len();
            if uncompressed_len >= threshold as usize {
                // Compress with zlib
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&uncompressed_buf)?;
                let compressed_data = encoder.finish()?;

                let mut packet_body = Vec::new();
                write_varint(&mut packet_body, uncompressed_len as i32)?;
                packet_body.extend_from_slice(&compressed_data);

                write_varint(&mut self.stream, packet_body.len() as i32)?;
                self.stream.write_all(&packet_body)?;
            } else {
                // Uncompressed
                let mut packet_body = Vec::new();
                write_varint(&mut packet_body, 0)?; // Data length = 0 means uncompressed
                packet_body.extend_from_slice(&uncompressed_buf);

                write_varint(&mut self.stream, packet_body.len() as i32)?;
                self.stream.write_all(&packet_body)?;
            }
        } else {
            let mut packet_body = Vec::new();
            write_varint(&mut packet_body, packet_id)?;
            packet_body.extend_from_slice(payload);

            write_varint(&mut self.stream, packet_body.len() as i32)?;
            self.stream.write_all(&packet_body)?;
        }
        self.stream.flush()?;
        Ok(())
    }
}
