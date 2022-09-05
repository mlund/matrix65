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
use std::error::Error;

mod filehost;
mod input;
mod io;
mod serial;
mod textui;

#[tokio::main]
async fn main() {
    if let Err(err) = do_main().await {
        println!("Error: {}", &err);
        std::process::exit(1);
    }
}

async fn do_main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let args = input::Args::parse();
    let mut port = serial::open_port(&args.port, args.baud)?;

    match args.command {
        input::Commands::Info {} => {
            serial::hypervisor_info(&mut port);
        }

        input::Commands::Reset { c64 } => {
            match c64 {
                true => serial::reset_to_c64(&mut port)?,
                false => serial::reset(&mut port)?,
            };
        }

        input::Commands::Type { text } => {
            serial::type_text(&mut port, text.as_str())?;
        }

        input::Commands::Prg { file, run } => {
            let (load_address, bytes) = match std::path::Path::new(&file).extension() {
                None => io::load_file_with_load_address(&file)?,
                Some(os_str) => match os_str.to_str() {
                    Some("prg") => io::load_file_with_load_address(&file)?,
                    Some("d81") => io::cbm_select_and_load(&file)?,
                    _ => panic!("invalid file extension"),
                },
            };
            match load_address {
                0x2001 => serial::write_memory(&mut port, load_address, &bytes)?,
                0x0801 => {
                    serial::reset_to_c64(&mut port)?;
                    serial::write_memory(&mut port, load_address, &bytes)?;
                },
                _ => todo!("arbitrary load address"),
            }
            if run {
                serial::type_text(&mut port, "run\r")?;
            }
        }

        input::Commands::Peek {
            address,
            length,
            outfile,
        } => {
            let bytes = serial::load_memory(&mut port, parse::<u32>(&address)?, length)?;
            match outfile {
                Some(name) => io::save_binary(&name, &bytes)?,
                None => io::hexdump(&bytes, 8),
            };
        }

        input::Commands::Filehost { dir } => {
            let entries = filehost::get_file_list().await?;
            textui::start_tui(&entries).unwrap();
            //entries?.iter().for_each(|entry| entry.print());
        }
    }
    Ok(())
}
