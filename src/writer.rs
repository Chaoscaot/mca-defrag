use std::collections::HashMap;
use std::io;
use std::io::{Cursor, Write};
use crate::parser::Chunks;

pub fn write_rca<W: Write>(chunks: Chunks, mut w: W) -> io::Result<()> {
    let mut c = HashMap::new();

    for chunk in chunks.chunks {
        c.insert(format!("{}:{}", chunk.location.x, chunk.location.z), chunk);
    }

    let mut header = [0u8; 1024 * 4];
    let mut header_buf = Cursor::new(&mut header[..]);

    let mut all_blocks = Vec::with_capacity(chunks.true_size);

    let mut offset = 2usize;

    for z in 0..32 {
        for x in 0..32 {
            let chunk = c.get(&format!("{}:{}", x, z)).unwrap();

            let mut location = offset;

            if chunk.location.size == 0 { location = 0 }

            let header_bytes = [
                (location >> 16) as u8,
                (location >> 8) as u8,
                location as u8,
                chunk.location.size as u8,
            ];

            offset += chunk.location.size;

            header_buf.write_all(&header_bytes)?;

            for block in &chunk.data {
                all_blocks.push(block);
            }
        }
    }

    w.write_all(&header)?;
    w.write_all(&chunks.timestamps)?;

    for block in all_blocks {
        w.write_all(block)?;
    }

    Ok(())
}