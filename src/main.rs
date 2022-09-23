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

use anyhow::Result;
use clap::Parser;
use matrix65::{filehost, io, serial};
use parse_int::parse;
use pretty_env_logger::env_logger::DEFAULT_FILTER_ENV;

mod input;
mod textui;

fn main() {
    if let Err(err) = do_main() {
        eprintln!("Error: {}", &err);
        std::process::exit(1);
    }
}

fn do_main() -> Result<()> {
    let args = input::Args::parse();

    if args.verbose && std::env::var(DEFAULT_FILTER_ENV).is_err() {
        std::env::set_var(DEFAULT_FILTER_ENV, "Debug");
    }
    pretty_env_logger::init();

    let mut port = serial::open_port(&args.port, args.baud)?;

    match args.command {
        input::Commands::Reset { c64 } => {
            serial::reset(&mut port)?;
            if c64 {
                serial::go64(&mut port)?
            }
        }

        input::Commands::Type { text } => {
            serial::type_text(&mut port, text.as_str())?;
        }

        input::Commands::Prg { file, reset, run } => {
            serial::handle_prg(&mut port, &file, reset, run)?;
        }

        input::Commands::Peek {
            address,
            length,
            outfile,
        } => {
            let bytes = serial::read_memory(&mut port, parse::<u32>(&address)?, length)?;
            match outfile {
                Some(name) => io::save_binary(&name, &bytes)?,
                None => matrix65::io::hexdump(&bytes, 8),
            };
        }

        input::Commands::Poke {
            address,
            file,
            value,
        } => {
            let bytes = match file.is_some() {
                true => matrix65::io::load_bytes(&file.unwrap())?,
                false => vec![value.ok_or(anyhow::Error::msg("VALUE required for poking"))?],
            };
            let parsed_address = parse::<u16>(&address)?;
            if parsed_address.checked_add(bytes.len() as u16 - 1).is_none() {
                // Merely a safety measure. Is this needed?
                return Err(anyhow::Error::msg(
                    "poking outside the 16-bit address space is currently unsupported",
                ));
            }
            matrix65::serial::write_memory(&mut port, parsed_address, &bytes)?;
        }

        input::Commands::Filehost {} => {
            let mut entries: Vec<_> = filehost::get_file_list()?
                .iter()
                .cloned()
                .filter(|item| {
                    item.filename.to_lowercase().ends_with(".prg")
                        | item.filename.to_lowercase().ends_with(".d81")
                })
                .collect();
            entries.sort_by_key(|i| i.title.clone());
            textui::terminal::start_tui(&mut port, &entries)?;
        }
    }
    Ok(())
}
