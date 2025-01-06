use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[derive(Debug, Copy, Clone)]
pub struct ChunkLocation {
    pub offset: usize,
    pub size: usize,
    pub x: u8,
    pub z: u8,
}

#[derive(Debug, Clone)]
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

pub fn parse_mca<R: Read + Seek>(f: &mut R) -> io::Result<Chunks> {
    let mut locations_header = [0u8; 1024 * 4];

    f.read_exact(&mut locations_header)?;

    let mut timestamp_header = [0u8; 1024 * 4];

    f.read_exact(&mut timestamp_header)?;

    let mut cursor = Cursor::new(&locations_header);

    let mut true_size = 0usize;

    let mut chunks = Vec::new();

    for z in 0..32 {
        for x in 0..32 {
            let mut chunk_location = [0u8; 3];
            let mut chunk_size = [0u8; 1];

            cursor.read_exact(&mut chunk_location)?;
            cursor.read_exact(&mut chunk_size)?;

            let offset = (chunk_location[0] as usize) << 16 | (chunk_location[1] as usize) << 8 | chunk_location[2] as usize;
            let size = chunk_size[0] as usize;

            true_size += size;

            f.seek(SeekFrom::Start(offset as u64 * 4096))?;

            let mut blocks = Vec::with_capacity(size);

            for _ in 0..size {
                let mut block = [0u8; 4096];
                f.read(&mut block[..])?;
                blocks.push(block);
            }

            chunks.push(Chunk {
                location: ChunkLocation {
                    offset,
                    size,
                    x: x as u8,
                    z: z as u8,
                },
                data: blocks,
            });
        }
    }

    Ok(Chunks {
        chunks,
        true_size,
        timestamps: timestamp_header,
    })
}
