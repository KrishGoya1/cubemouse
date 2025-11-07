use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub enum Packet {
    Move { dx: i16, dy: i16 },
    // Click { ... }, Scroll { ... } future
}

/// Parses a single binary packet
pub fn parse_packet(buf: &[u8]) -> Option<Packet> {
    if buf.len() < 1 { return None; }

    let opcode = buf[0];
    match opcode {
        0x01 => { // MOVE
            if buf.len() < 5 { return None; } // 1 byte opcode + 4 bytes dx/dy
            let mut rdr = Cursor::new(&buf[1..]);
            let dx = rdr.read_i16::<LittleEndian>().ok()?;
            let dy = rdr.read_i16::<LittleEndian>().ok()?;
            Some(Packet::Move { dx, dy })
        }
        _ => None,
    }
}
