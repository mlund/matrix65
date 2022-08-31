use log::info;
use serialport::SerialPort;
use std::time::Duration;

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

/// Get MEGA65 info
pub fn hypervisor_info(port: &mut Box<dyn SerialPort>) {
    info!("Requesting serial monitor info");
    let mut buffer = String::new();
    port.write_all("h\n".as_bytes()).expect("Write failed!");
    port.read_to_string(&mut buffer)
        .expect("Serial read error - likely non-unicode data");
    print!("{}", buffer);
}
