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

/// Routines related to file loading and stdio
use cbm::disk;
use cbm::disk::file::FileOps;
use log::info;
use std::fs::File;
use std::io::{self, Read, Write};
use tempfile::Builder;

/// Load file or url into byte vector
fn load_bytes(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if filename.starts_with("http") {
        bytes = reqwest::blocking::get(filename)
            .unwrap()
            .bytes()
            .unwrap()
            .to_vec();
    } else {
        File::open(&filename)?.read_to_end(&mut bytes)?;
    }
    assert!(bytes.len() < 0xffff);
    Ok(bytes)
}

/// Load PRG from prg and CBM disk files
///
/// If an archive (.d64|.d81) is detected, the user is presented with a selection
/// of found PRG files. Returns intended load address and raw bytes.
pub fn load_prg(file: &str) -> std::io::Result<(u16, Vec<u8>)> {
    match std::path::Path::new(&file).extension() {
        None => load_with_load_address(&file),
        Some(os_str) => match os_str.to_ascii_lowercase().to_str() {
            Some("prg") => load_with_load_address(&file),
            Some("d81") | Some("d64") => cbm_select_and_load(&file),
            _ => {
                let error =
                    std::io::Error::new(std::io::ErrorKind::Other, "invalid file extension");
                Err(error)
            }
        },
    }
}

/// Purge and return load address from vector of bytes
///
/// The two first bytes form the 16-bit load address, little endian.
/// Returns found address and removes the first two bytes from the byte vector.
fn purge_load_address(bytes: &mut Vec<u8>) -> u16 {
    let load_address = u16::from_le_bytes(
        bytes[0..2]
            .try_into()
            .expect("error extracting load address"),
    );
    *bytes = bytes[2..].to_vec();
    load_address
}

/// Open a CBM diskimage from file or url
fn cbm_open(diskimage: &str) -> std::io::Result<Box<dyn cbm::disk::Disk>> {
    if diskimage.starts_with("http") {
        let tmp_dir = Builder::new().tempdir()?;
        let path = tmp_dir.path().join("temp-image");
        let filename = path.to_str().unwrap();
        let bytes = reqwest::blocking::get(diskimage)
            .unwrap()
            .bytes()
            .unwrap()
            .to_vec();
        save_binary(filename, &bytes);
        return disk::open(filename, false);
    }
    disk::open(diskimage, false)
}

/// User select PRG file from CBM image file or url
///
/// Looks for PRG files on the CBM disk image and
/// presents a numbered list from which the user
/// can select. Loads the file and returns the load
/// address together with raw bytes.
fn cbm_select_and_load(diskimage: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let disk = cbm_open(diskimage)?;
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
    io::stdin().read_line(&mut selection)?;
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

/// Load a prg file or url into a byte vector and detect load address
fn load_with_load_address(filename: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let mut bytes = load_bytes(filename)?;
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
