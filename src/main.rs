use clap::Parser;
use log::info;
use std::io::Read;

mod input;
mod serial;

// /dev/cu.usbserial-AQ027F6E

fn main() {
    pretty_env_logger::init();
    let args = input::Args::parse();
    let mut port = serial::open_port(&args.port, args.baud);

    match args.command {
        input::Commands::List {} => {
            serial::print_ports();
        }

        input::Commands::Info {} => {
            serial::hypervisor_info(&mut port);
        }

        input::Commands::Reset {} => {
            serial::reset(&mut port).unwrap();
        }

        input::Commands::Type { text } => {
            serial::type_text(&mut port, &text.as_str());
        }

        input::Commands::Prg { file, run } => {
            let (load_address, bytes) = load_file_with_load_address(file);
            match load_address {
                0x2001 => serial::load_memory(&mut port, load_address, &bytes),
                0x0801 => todo!("c64 load address"),
                _ => todo!("arbitrary load address"),
            }
            if run {
                serial::type_text(&mut port, "run\r");
            }
        }
    }
}

/// Load a prg file into a vector
///
/// Returns intended load address and raw bytes
fn load_file_with_load_address(filename: String) -> (u16, Vec<u8>) {
    let mut bytes = Vec::new();
    std::fs::File::open(&filename)
        .expect("could not open file")
        .by_ref()
        .read_to_end(&mut bytes)
        .unwrap();
    // the first two bytes form the 16-bit load address, little endian
    let load_address = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    info!(
        "Read {} bytes from {}; load address = 0x{:x}",
        bytes.len(),
        &filename,
        load_address
    );
    bytes.remove(0); // yikes...
    bytes.remove(1);
    (load_address, bytes)
}
