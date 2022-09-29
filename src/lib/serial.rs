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

//! Routines for serial communication with MEGA65

use crate::LoadAddress;

use super::io;
use anyhow::Result;
use hex::FromHex;
use log::debug;
use serialport::SerialPort;
use std::thread;
use std::time::Duration;

/// Delay after writing to serial port
const DELAY_WRITE: Duration = Duration::from_millis(20);
/// Delay between sending key presses
const DELAY_KEYPRESS: Duration = DELAY_WRITE;
/// Default serial speed in bits per second
pub const DEFAULT_BAUD_RATE: u32 = 2000000;

/// Stop the MEGA65 CPU
pub fn stop_cpu(port: &mut Box<dyn SerialPort>) -> Result<()> {
    port.write_all("t1\r".as_bytes())?;
    port.flush()?;
    thread::sleep(DELAY_WRITE);
    Ok(())
}

/// Start the MEGA65 CPU after being halted
pub fn start_cpu(port: &mut Box<dyn SerialPort>) -> Result<()> {
    port.write_all("t0\r".as_bytes())?;
    port.flush()?;
    thread::sleep(DELAY_WRITE);
    Ok(())
}

/// Detect if in C65 mode
pub fn is_c65_mode(port: &mut Box<dyn SerialPort>) -> Result<bool> {
    let byte = read_memory(port, 0xffd3030, 1)?[0];
    Ok(byte == 0x64)
}

/// Print available serial ports
fn print_ports() {
    debug!("Detecting serial ports");
    serialport::available_ports()
        .expect("No serial ports found!")
        .iter()
        .for_each(|port| println!("{}", port.port_name));
    println!();
}

/// Open serial port - show available ports and stop if invalid
pub fn open_port(name: &str, baud_rate: u32) -> Result<Box<dyn SerialPort>> {
    debug!("Opening serial port {}", name);
    match serialport::new(name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(port) => Ok(port),
        Err(err) => {
            eprintln!("Invalid serial port, try one of these?\n");
            print_ports();
            Err(err.into())
        }
    }
}

/// Reset the MEGA65
pub fn reset(port: &mut Box<dyn SerialPort>) -> Result<()> {
    debug!("Sending RESET signal");
    port.write_all("!\n".as_bytes())?;
    thread::sleep(Duration::from_secs(4));
    Ok(())
}

/// If not already there, go to C64 mode via key presses
pub fn go64(port: &mut Box<dyn SerialPort>) -> Result<()> {
    debug!("Sending GO64");
    if is_c65_mode(port)? {
        type_text(port, "go64\ry\r")?;
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

/// If not already there, go to C65 mode via a reset
pub fn go65(port: &mut Box<dyn SerialPort>) -> Result<()> {
    if !is_c65_mode(port)? {
        reset(port)?;
    }
    Ok(())
}

/// Translate and type a single letter on MEGA65
fn type_key(port: &mut Box<dyn SerialPort>, mut key: char) -> Result<()> {
    let mut c1: u8 = 0x7f;
    let mut c2 = match key {
        '!' => {
            key = '1';
            0x0f
        }
        '\"' => {
            key = '2';
            0x0f
        }
        '#' => {
            key = '3';
            0x0f
        }
        '$' => {
            key = '4';
            0x0f
        }
        '%' => {
            key = '5';
            0x0f
        }
        '(' => {
            key = '8';
            0x0f
        }
        ')' => {
            key = '9';
            0x0f
        }
        '?' => {
            key = '/';
            0x0f
        }
        '<' => {
            key = ',';
            0x0f
        }
        '>' => {
            key = '.';
            0x0f
        }
        _ => 0x7f,
    };

    match key as u8 {
        0x14 => c1 = 0x00, // INST/DEL
        0x0d => c1 = 0x01, // Return
        0x1d => c1 = 0x02, // Cursor right
        0xf7 => c1 = 0x03,
        0x9d => {
            // Cursor left
            c1 = 0x02;
            c2 = 0x0f;
        }
        0x91 => {
            // Cursor up
            c1 = 0x07;
            c2 = 0x0f;
        }
        0xf1 => c2 = 0x04, // F1
        0xf3 => c1 = 0x05, // F3
        0xf5 => c1 = 0x06, // F5
        0x11 => c1 = 0x07, // Cursor down
        b'3' => c1 = 0x08,
        b'w' => c1 = 0x09,
        b'a' => c1 = 0x0a,
        b'4' => c1 = 0x0b,
        b'z' => c1 = 0x0c,
        b's' => c1 = 0x0d,
        b'e' => c1 = 0x0e,
        b'5' => c1 = 0x10,
        b'r' => c1 = 0x11,
        b'd' => c1 = 0x12,
        b'6' => c1 = 0x13,
        b'c' => c1 = 0x14,
        b'f' => c1 = 0x15,
        b't' => c1 = 0x16,
        b'x' => c1 = 0x17,
        b'7' => c1 = 0x18,
        b'y' => c1 = 0x19,
        b'g' => c1 = 0x1a,
        b'8' => c1 = 0x1b,
        b'b' => c1 = 0x1c,
        b'h' => c1 = 0x1d,
        b'u' => c1 = 0x1e,
        b'v' => c1 = 0x1f,
        b'9' => c1 = 0x20,
        b'i' => c1 = 0x21,
        b'j' => c1 = 0x22,
        b'0' => c1 = 0x23,
        b'm' => c1 = 0x24,
        b'k' => c1 = 0x25,
        b'o' => c1 = 0x26,
        b'n' => c1 = 0x27,
        b'+' => c1 = 0x28,
        b'p' => c1 = 0x29,
        b'l' => c1 = 0x2a,
        b'-' => c1 = 0x2b,
        b'.' => c1 = 0x2c,
        b':' => c1 = 0x2d,
        b'@' => c1 = 0x2e,
        b',' => c1 = 0x2f,
        b'}' => c1 = 0x30,
        b'*' => c1 = 0x31,
        b';' => c1 = 0x32,
        0x13 => c1 = 0x33,
        b'=' => c1 = 0x35,
        b'/' => c1 = 0x37,
        b'1' => c1 = 0x38,
        b'_' => c1 = 0x39,
        b'2' => c1 = 0x3b,
        b' ' => c1 = 0x3c,
        b'q' => c1 = 0x3e,
        0x03 => c1 = 0x3f, // RUN/STOP
        0x0c => c1 = 0x3f,
        _ => c1 = 0x7f,
    }

    port.write_all(format!("sffd3615 {:02x} {:02x}\n", c1, c2).as_bytes())?;
    thread::sleep(DELAY_KEYPRESS);
    Ok(())
}

/// Call this when done typing
fn stop_typing(port: &mut Box<dyn SerialPort>) -> Result<()> {
    port.write_all("sffd3615 7f 7f 7f \n".as_bytes())?;
    thread::sleep(DELAY_WRITE);
    Ok(())
}

/// Send array of key presses
pub fn type_text(port: &mut Box<dyn SerialPort>, text: &str) -> Result<()> {
    // Manually translate user defined escape codes:
    // https://stackoverflow.com/questions/72583983/interpreting-escape-characters-in-a-string-read-from-user-input
    debug!("Typing text");
    thread::sleep(DELAY_KEYPRESS);
    text.replace("\\r", "\r")
        .replace("\\n", "\r")
        .chars()
        .for_each(|key| type_key(port, key).unwrap_or(()));
    stop_typing(port)?;
    Ok(())
}

/// Get MEGA65 info (@todo under construction)
#[allow(dead_code)]
fn mega65_info(port: &mut Box<dyn SerialPort>) -> Result<()> {
    debug!("Requesting serial monitor info");
    port.write_all("h\n".as_bytes())?;
    thread::sleep(DELAY_WRITE);

    let mut buffer = Vec::new();
    buffer.resize(65, 0);
    port.read_exact(&mut buffer)?;
    let lines = buffer.split(|i| *i == b'\n');
    for line in lines {
        for i in line {
            print!("{}", *i as char);
        }
    }
    println!();
    Ok(())
}

/// Load memory from MEGA65 starting at given address
pub fn read_memory(port: &mut Box<dyn SerialPort>, address: u32, length: usize) -> Result<Vec<u8>> {
    debug!("Loading {} bytes from 0x{:x}", length, address);
    flush_monitor(port)?;
    stop_cpu(port)?;
    // request memory dump (MEMORY, "M" command)
    port.write_all(format!("m{:07x}\r", address).as_bytes())?;
    thread::sleep(DELAY_WRITE);

    let mut buffer = Vec::new();
    let mut bytes = Vec::new();
    bytes.reserve(length);

    // skip header
    buffer.resize(27, 0);
    port.read_exact(&mut buffer)?;

    while bytes.len() < length {
        // load 16 two-letter byte codes
        buffer.resize(16 * 2, 0);
        port.read_exact(&mut buffer)?;
        // convert two-letter codes to bytes
        let mut sixteen_bytes: Vec<u8> = Vec::from_hex(&buffer)?;
        bytes.append(&mut sixteen_bytes);
        // trigger next memory dump and ignore header
        port.write_all("m\r".as_bytes())?;
        thread::sleep(DELAY_WRITE);
        buffer.resize(18, 0);
        port.read_exact(&mut buffer)?;
    }
    bytes.truncate(length);
    start_cpu(port)?;
    Ok(bytes)
}

/// Try to empty the monitor by reading one byte until nothing more can be read
///
/// There must be more elegant ways to do this...
pub fn flush_monitor(port: &mut Box<dyn SerialPort>) -> Result<()> {
    port.write_all(&[0x15, b'#', b'\r'])?;
    let mut byte = [0u8];
    loop {
        thread::sleep(DELAY_WRITE);
        match port.read_exact(&mut byte) {
            Ok(()) => continue,
            Err(_) => break,
        }
    }
    Ok(())
}

/// Write bytes to MEGA65 at 200 kB/s at default baud rate
pub fn write_memory(port: &mut Box<dyn SerialPort>, address: u16, bytes: &[u8]) -> Result<()> {
    debug!("Writing {} byte(s) to address 0x{:x}", bytes.len(), address);
    stop_cpu(port)?;
    port.write_all(format!("l{:x} {:x}\r", address, address + bytes.len() as u16).as_bytes())?;
    thread::sleep(DELAY_WRITE);
    port.write_all(bytes)?;
    thread::sleep(DELAY_WRITE);
    start_cpu(port)?;
    Ok(())
}

/// Transfer to MEGA65 and optionally run PRG
///
/// C64/C65 modes are selected from the load address
pub fn handle_prg_from_bytes(
    port: &mut Box<dyn SerialPort>,
    bytes: &[u8],
    load_address: LoadAddress,
    reset_before_run: bool,
    run: bool,
) -> Result<()> {
    if reset_before_run {
        reset(port)?;
    }
    match load_address {
        LoadAddress::Commodore65 => go65(port)?,
        LoadAddress::Commodore64 => go64(port)?,
        _ => {
            return Err(anyhow::Error::msg("unsupported load address"));
        }
    }
    write_memory(port, load_address.value(), bytes)?;
    if run {
        type_text(port, "run\r")?;
    }
    Ok(())
}

/// Transfers and optionally run PRG to MEGA65
///
/// Here `file` can be a local file or a url. CBM disk images are allowed and
/// C64/C65 modes are detected from load address.
pub fn handle_prg(
    port: &mut Box<dyn SerialPort>,
    file: &str,
    reset_before_run: bool,
    run: bool,
) -> Result<()> {
    let (load_address, bytes) = io::load_prg(file)?;
    handle_prg_from_bytes(port, &bytes, load_address, reset_before_run, run)
}
