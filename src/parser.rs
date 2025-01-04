use std::io;
use std::io::{Cursor, Read};

#[derive(Debug)]
pub struct ChunkLocation {
    pub offset: usize,
    pub size: usize,
    pub x: u8,
    pub z: u8,
}

#[derive(Debug)]
pub struct Chunk {
    pub location: ChunkLocation,
    pub data: Vec<[u8; 4096]>,
}

#[derive(Debug)]
pub struct Chunks {
    pub chunks: Vec<Chunk>,
    pub true_size: usize,
    pub timestamps: [u8; 4096],
}

pub fn parse_mca<R: Read>(f: &mut R) -> io::Result<Chunks> {
    let mut locations_header = [0u8; 1024 * 4];

    f.read_exact(&mut locations_header)?;

    let mut timestamp_header = [0u8; 1024 * 4];

    f.read_exact(&mut timestamp_header)?;

    let mut cursor = Cursor::new(&locations_header);

    let mut chunk_locations = Vec::new();

    let mut true_size = 0usize;

    for z in 0..32 {
        for x in 0..32 {
            let mut chunk_location = [0u8; 3];
            let mut chunk_size = [0u8; 1];

            cursor.read_exact(&mut chunk_location)?;
            cursor.read_exact(&mut chunk_size)?;

            let offset = (chunk_location[0] as usize) << 16 | (chunk_location[1] as usize) << 8 | chunk_location[2] as usize;
            let size = chunk_size[0] as usize;

            true_size += size;

            chunk_locations.push(ChunkLocation {
                offset,
                size,
                x,
                z,
            })
        }
    }

    let mut chunks = Vec::new();

    chunk_locations.sort_by(|a, b| a.offset.cmp(&b.offset));

    for chunk_location in chunk_locations {
        let mut blocks = Vec::with_capacity(chunk_location.size);

        for _ in 0..chunk_location.size {
            let mut chunk_data = [0u8; 4096];
            f.read_exact(&mut chunk_data)?;
            blocks.push(chunk_data);
        }

        chunks.push(Chunk {
            location: chunk_location,
            data: blocks,
        });
    }

    Ok(Chunks {
        chunks,
        true_size,
        timestamps: timestamp_header,
    })
}
