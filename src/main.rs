mod parser;
mod writer;

use std::fs::File;
use std::io;
use std::io::{Cursor, Write};
use clap::{Arg, ArgAction, Command, ValueHint};
use crate::parser::Chunks;

fn main() {
    let cmd = Command::new("rca-defrag")
        .about("CLI tool for defragging Minecraft .rca Regon Files")
        .arg(Arg::new("input")
            .required(true)
            .action(ArgAction::Append)
            .trailing_var_arg(true)
            .help("The input file to defrag")
            .value_hint(ValueHint::FilePath))
        .arg(Arg::new("dry")
            .long("dry")
            .short('d')
            .help("Dry run, don't write the output")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("quiet")
            .long("quiet")
            .short('q')
            .help("Prints less information")
            .action(ArgAction::SetTrue));

    let matches = cmd.get_matches();

    let quiet = matches.get_flag("quiet");
    let dry = matches.get_flag("dry");

    let mut total_read = 0u64;
    let mut total_written = 0u64;

    let start = std::time::Instant::now();

    for x in matches.get_many::<String>("input").unwrap() {
        let result = defrag(x, quiet, dry);
        if let Err(e) = result {
            eprintln!("Error: {}", e);
        } else if let Ok((read, written)) = result {
            total_read += read;
            total_written += written;

        }
    }

    let elapsed = start.elapsed();

    if !quiet {
        println!("Saved {:.1}MiB, ({:.1}%) in {}ms",
                 (total_read - total_written) as f32 / 1024f32 / 1024f32,
                 (total_read - total_written) as f32 / total_read as f32 * 100f32,
                 elapsed.as_millis()
        );
    }
}

mod test {
    use crate::defrag;

    #[test]
    fn test_defrag() {
        defrag(&"r.0.-1.mca".to_string(), false, true).unwrap();
    }
}

fn defrag(path: &String, quiet: bool, dry: bool) -> io::Result<(u64, u64)> {
    let start = if !quiet {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let chunks = read(path)?;
    let len = write(chunks.chunks, path, chunks.initial_size, dry)?;

    let saved = (chunks.initial_size - len) as f32 / 1024f32 / 1024f32;

    if !quiet && saved > 1f32 {
        let elapsed = start.unwrap().elapsed();

        println!("{}: Saved {:.1}MiB, ({:.1}%) in {}ms",
                 path,
                 saved,
                 (chunks.initial_size - len) as f32 / chunks.initial_size as f32 * 100f32,
                 elapsed.as_millis()
        );
    }

    Ok((
        chunks.initial_size,
        len
        ))
}

fn read(path: &String) -> io::Result<ReadResult> {
    let mut file = File::open(path)?;

    let initial_size = file.metadata()?.len();

    Ok(ReadResult {
        chunks: parser::parse_mca(&mut file)?,
        initial_size
    })
}

fn write(chunks: Chunks, path: &String, initial_size: u64, dry: bool) -> io::Result<u64> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024 + 4098 * chunks.true_size);
    let cursor = Cursor::new(&mut buf);

    writer::write_rca(chunks, cursor)?;

    let mut last_zeros = 0u64;

    for i in (0..buf.len()).rev() {
        if buf[i] == 0 {
            last_zeros += 1;
        } else {
            break;
        }
    }

    let len = buf.len() as u64 - last_zeros;

    if len >= initial_size {
        return Ok(initial_size)
    }

    if dry {
        return Ok(len)
    }

    let mut out = File::create(path)?;

    out.set_len(len)?;

    out.write_all(&buf)?;

    Ok(len)
}

struct ReadResult {
    chunks: Chunks,
    initial_size: u64,
}
