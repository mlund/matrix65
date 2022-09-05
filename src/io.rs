// copyright 2022 mikael lund aka wombat
//
// licensed under the apache license, version 2.0 (the "license");
// you may not use this file except in compliance with the license.
// you may obtain a copy of the license at
//
//     http://www.apache.org/licenses/license-2.0
//
// unless required by applicable law or agreed to in writing, software
// distributed under the license is distributed on an "as is" basis,
// without warranties or conditions of any kind, either express or implied.
// see the license for the specific language governing permissions and
// limitations under the license.

use cbm::disk;
use log::info;
use std::fs::File;
use cbm::disk::file::FileOps;
use std::io::{self, Read, Write};

/// Load file into byte vector
pub fn load_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(&filename).unwrap().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Purge and return load address from vector of bytes
///
/// The two first bytes form the 16-bit load address, little endian.
/// Returns found address and removes the first two bytes from the byte vector.
fn purge_load_address(bytes: &mut Vec<u8>) -> u16 {
    let load_address = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    *bytes = bytes[2..].to_vec();
    load_address
}

/// User select PRG file from image
/// 
/// Looks for PRG files on the CBM disk image and
/// presents a numbered list from which the user
/// can select. Loads the file and returns the load
/// address as well as the raw bytes.
pub fn cbm_select_and_load(diskimage: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let disk = disk::open(diskimage, false)?;
    let dir = disk.directory()?;
    let prg_files = &mut dir
        .iter()
        .filter(|entry| entry.file_attributes.file_type == cbm::disk::directory::FileType::PRG);
    for (counter, file) in prg_files.clone().enumerate() {
        println!("[{}] {}.prg", counter, file.filename.to_string());
    }
    print!("Select: ");
    io::stdout().flush()?;
    let mut selection = String::new();
    io::stdin()
        .read_line(&mut selection)
        .expect("Failed to read input");
    let index = selection
        .trim_end()
        .parse::<usize>()
        .expect("invalid index selection");
    let entry = prg_files.nth(index).expect("invalid file");
    let mut bytes = Vec::<u8>::new();
    disk.open_file(&entry.filename)?
        .reader()?
        .read_to_end(&mut bytes)?;
    let load_address = purge_load_address(&mut bytes);
    Ok((load_address, bytes))
}

/// Load a prg file into a byte vector and detect load address
pub fn load_file_with_load_address(filename: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let mut bytes = load_file(filename)?;
    let load_address = purge_load_address(&mut bytes);
    info!(
        "Read {} bytes from {}; detected load address = 0x{:x}",
        bytes.len() + 2,
        &filename,
        load_address
    );
    Ok((load_address, bytes.to_vec()))
}

/// Save bytes to binary file
pub fn save_binary(filename: &str, bytes: &[u8]) -> std::io::Result<()> {
    info!("Saving {} bytes to {}", bytes.len(), filename);
    File::create(filename)?.write_all(bytes)
}

/// Print bytes to screen
pub fn hexdump(bytes: &[u8], bytes_per_line: usize) {
    let to_hex = |i: u8| format!("0x{:02x}", i);
    bytes.chunks(bytes_per_line).for_each(|line| {
        for byte in line {
            print!("{} ", to_hex(*byte));
        }
        println!();
    });
}
