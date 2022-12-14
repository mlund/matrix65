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
use matrix65::{filehost, serial};
use pretty_env_logger::env_logger::DEFAULT_FILTER_ENV;

mod commands;
mod input;
mod repl;
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
        input::Commands::Reset { c64 } => commands::reset(&mut port, c64)?,
        input::Commands::Filehost {} => commands::filehost(&mut port)?,
        input::Commands::Cmd {} => repl::start_repl(&mut port)?,
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
            disassemble,
        } => commands::peek(&mut port, address, length, outfile, disassemble)?,

        input::Commands::Poke {
            address,
            file,
            value,
        } => commands::poke(file, value, address, &mut port)?,
    }
    Ok(())
}
