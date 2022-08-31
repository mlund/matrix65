use log::info;
use serialport::SerialPort;
use std::thread;
use std::time::Duration;

/// Delay between sending key presses
const DELAY_KEYPRESS: Duration = Duration::from_micros(20000);

/// Print available serial ports
pub fn print_ports() {
    info!("Detecting serial ports.");
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

fn stop_typing(port: &mut Box<dyn SerialPort>) {
    port.write_all("sffd3615 7f 7f 7f \n".as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

/// Send array of key presses
pub fn type_text(port: &mut Box<dyn SerialPort>, text: &str) {
    // Manually translate user defined escape codes:
    // https://stackoverflow.com/questions/72583983/interpreting-escape-characters-in-a-string-read-from-user-input
    for key in text.replace("\\r", "\r").chars() {
        type_key(port, key);
    }
    stop_typing(port);
}

/// Get MEGA65 info
pub fn hypervisor_info(port: &mut Box<dyn SerialPort>) {
    info!("Requesting serial monitor info");
    let mut buffer = String::new();
    port.write_all("h\n".as_bytes()).expect("Write failed!");
    port.read_to_string(&mut buffer)
        .expect("Serial read error - likely non-unicode data");
    print!("{}", buffer);
}
