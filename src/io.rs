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

/// File I/O

use log::info;
use std::io::Read;

/// Load file into byte vector
pub fn load_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(&filename)?
        .by_ref()
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Load a prg file into a vector and detect load address
///
/// The two bytes form the 16-bit load address, little endian.
/// Returns intended load address and raw bytes (excluding the first two bytes)
pub fn load_file_with_load_address(filename: &str) -> std::io::Result<(u16, Vec<u8>)> {
    let bytes = load_file(filename)?;
    let load_address = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    info!(
        "Read {} bytes from {}; detected load address = 0x{:x}",
        bytes.len(),
        &filename,
        load_address
    );
    Ok((load_address, bytes[2..].to_vec()))
}
