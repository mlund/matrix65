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

use hex::FromHex;
use log::info;
use serialport::SerialPort;
use std::thread;
use std::time::Duration;

/// Delay between sending key presses
const DELAY_KEYPRESS: Duration = Duration::from_micros(20000);

fn stop_cpu(port: &mut Box<dyn SerialPort>) {
    port.flush().unwrap();
    port.write_all("t1\r".as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

fn start_cpu(port: &mut Box<dyn SerialPort>) {
    port.flush().unwrap();
    port.write_all("t0\r".as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

/// Print available serial ports
pub fn print_ports() {
    info!("Detecting serial ports");
    let ports = serialport::available_ports().expect("No ports found!");
    for port in ports {
        println!("{}", port.port_name);
    }
}

/// Open serial port - panic on failure
pub fn open_port(name: &String, baud_rate: u32) -> Box<dyn SerialPort> {
    info!("Opening serial port {}", name);
    serialport::new(name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port")
}

/// Reset the MEGA65
pub fn reset(port: &mut Box<dyn SerialPort>) -> Result<(), std::io::Error> {
    info!("Sending RESET signal");
    port.write_all("!\n".as_bytes())
}

/// Translate and type a single letter on MEGA65
fn type_key(port: &mut Box<dyn SerialPort>, mut key: char) {
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

    port.write_all(format!("sffd3615 {:02x} {:02x}\n", c1, c2).as_bytes())
        .unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

/// Call this when done typing
fn stop_typing(port: &mut Box<dyn SerialPort>) {
    port.write_all("sffd3615 7f 7f 7f \n".as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

/// Send array of key presses
pub fn type_text(port: &mut Box<dyn SerialPort>, text: &str) {
    // Manually translate user defined escape codes:
    // https://stackoverflow.com/questions/72583983/interpreting-escape-characters-in-a-string-read-from-user-input
    info!("Typing text");
    for key in text.replace("\\r", "\r").replace("\\n", "\r").chars() {
        type_key(port, key);
    }
    stop_typing(port);
}

/// Get MEGA65 info
#[allow(dead_code)]
pub fn hypervisor_info(port: &mut Box<dyn SerialPort>) {
    info!("Requesting serial monitor info");
    port.write_all("h\n".as_bytes()).expect("Write failed!");
    thread::sleep(DELAY_KEYPRESS);
    let mut buffer = Vec::new();
    buffer.resize(1024, 0);
    let n = port
        .read(&mut buffer)
        .expect("Serial read error - likely non-unicode data");
    println!("{}", n);

    for i in buffer {
        if i.is_ascii() {
            print!("{}", i as char);
        }
    }
}

/// Loads memory from MEGA65 starting at given address
pub fn load_memory(port: &mut Box<dyn SerialPort>, address: u32, length: usize) -> Vec<u8> {
    info!("Loading {} bytes from 0x{:x}", length, address);
    stop_cpu(port);
    // request memory dump (MEMORY, "M" command)
    port.write_all(format!("m{:x}\r", address).as_bytes())
        .unwrap();
    thread::sleep(DELAY_KEYPRESS);

    let mut buffer = Vec::new();
    let mut bytes = Vec::new();
    bytes.reserve(length);

    // skip header
    buffer.resize(24, 0);
    port.read_exact(&mut buffer).unwrap();

    while bytes.len() < length {
        // load 16 two-letter byte codes
        buffer.resize(16 * 2, 0);
        port.read_exact(&mut buffer).expect("buffer load error");
        // convert two-letter codes to bytes
        let mut sixteen_bytes: Vec<u8> = Vec::from_hex(&buffer).expect("invalid hex");
        bytes.append(&mut sixteen_bytes);
        // trigger next memory dump and ignore header
        port.write_all(format!("m\r").as_bytes()).unwrap();
        thread::sleep(DELAY_KEYPRESS);
        buffer.resize(18, 0);
        port.read_exact(&mut buffer).expect("buffer load error");
    }
    start_cpu(port);
    bytes.truncate(length);
    bytes
}

/// Copy chunks of data to MEGA65 at 200 kB/s at default baud rate
pub fn write_memory(port: &mut Box<dyn SerialPort>, address: u16, bytes: &[u8]) {
    info!("Loading {} bytes to address 0x{:x}", bytes.len(), address);
    stop_cpu(port);
    port.write_all(format!("l{:x} {:x}\r", address, address + bytes.len() as u16).as_bytes())
        .unwrap();
    port.write_all(bytes).unwrap();
    start_cpu(port);
}