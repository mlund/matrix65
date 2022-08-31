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

use clap::Parser;
use parse_int::parse;

mod input;
mod io;
mod serial;

fn main() {
    pretty_env_logger::init();
    let args = input::Args::parse();
    let mut port = serial::open_port(&args.port, args.baud);

    match args.command {
        input::Commands::List {} => {
            serial::print_ports();
        }

        input::Commands::Info {} => {
            todo!("coming soon");
            //serial::hypervisor_info(&mut port);
        }

        input::Commands::Reset {} => {
            serial::reset(&mut port).unwrap();
        }

        input::Commands::Type { text } => {
            serial::type_text(&mut port, text.as_str());
        }

        input::Commands::Prg { file, run } => {
            let (load_address, bytes) = io::load_file_with_load_address(&file).expect("load error");
            match load_address {
                0x2001 => serial::write_memory(&mut port, load_address, &bytes),
                0x0801 => todo!("c64 load address"),
                _ => todo!("arbitrary load address"),
            }
            if run {
                serial::type_text(&mut port, "run\r");
            }
        }

        input::Commands::Peek { address, length } => {
            let bytes = serial::load_memory(&mut port, parse::<u32>(&address).unwrap(), length);
            for byte in bytes {
                print!("{:02x} ", byte);
            }
        }
    }
}
