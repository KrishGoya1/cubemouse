use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub enum Packet {
    Move { dx: i16, dy: i16 },
    LeftClick,
    RightClick,
    Scroll { dy: i16 },
}

pub fn parse_packet(buf: &[u8]) -> Option<Packet> {
    if buf.len() < 1 { return None; }

    match buf[0] {
        0x01 => { // MOVE
            if buf.len() < 5 { return None; }
            let mut rdr = Cursor::new(&buf[1..]);
            let dx = rdr.read_i16::<LittleEndian>().ok()?;
            let dy = rdr.read_i16::<LittleEndian>().ok()?;
            Some(Packet::Move { dx, dy })
        }
        0x02 => Some(Packet::LeftClick),
        0x03 => Some(Packet::RightClick),
        0x04 => { // SCROLL (dy only)
            if buf.len() < 3 { return None; }
            let mut rdr = Cursor::new(&buf[1..]);
            let dy = rdr.read_i16::<LittleEndian>().ok()?;
            Some(Packet::Scroll { dy })
        }
        _ => None,
    }
}
