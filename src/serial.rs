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

pub fn type_key(port: &mut Box<dyn SerialPort>, key: char) {
    let mut c1 : u8 = 0x7f;
    let mut c2 : u8 = 0x7f;
    port.write_all(format!("sffd3615 {:02x} {:02x}\n", c1 as u8, c2 as u8).as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

pub fn stop_typing(port: &mut Box<dyn SerialPort>) {
    port.write_all("sffd3615 7f 7f 7f \n".as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
}

/// Reset the MEGA65
pub fn type_run(port: &mut Box<dyn SerialPort>) {
    info!("Typing RUN");
    port.write_all(format!("sffd3615 {:02x} {:02x}\n", 0x11, 0x7f).as_bytes()).unwrap();

    thread::sleep(DELAY_KEYPRESS);

    port.write_all(format!("sffd3615 {:02x} {:02x}\n", 0x1e, 0x7f).as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
    port.write_all(format!("sffd3615 {:02x} {:02x}\n", 0x27, 0x7f).as_bytes()).unwrap();
    thread::sleep(DELAY_KEYPRESS);
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
