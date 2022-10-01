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

use clap::{Parser, Subcommand};
use matrix65::serial::DEFAULT_BAUD_RATE;

/// Matrix Mode Serial Communicator for MEGA65
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Transfer and run PRG from file or archive
    #[clap(arg_required_else_help = true)]
    Prg {
        /// File/URL to load or scan (.prg|.d64|.d81)
        #[clap(value_parser)]
        file: String,
        /// Reset before loading
        #[clap(long, action)]
        reset: bool,
        /// Run after loading
        #[clap(long, short = 'r', action)]
        run: bool,
    },

    /// Send key presses
    #[clap(arg_required_else_help = true)]
    Type {
        /// Text to type - use \r for return
        #[clap(value_parser)]
        text: String,
    },

    /// Reset MEGA65
    Reset {
        /// Reset into C64 mode
        #[clap(long, action)]
        c64: bool,
    },

    /// Peek into memory
    #[clap(arg_required_else_help = true)]
    Peek {
        /// Address to peek into, e.g. 4096 (dec) or 0x1000 (hex)
        #[clap(long, short = '@')]
        address: String,
        /// Number of bytes to retrieve
        #[clap(long = "num", short = 'n', default_value_t = 1)]
        length: usize,
        /// Output to binary file instead of hexdump
        #[clap(long, short = 'o')]
        outfile: Option<String>,
        /// Disassemble instead of hexdump (currently only 6502)
        #[clap(long = "dasm", short = 'd', action, conflicts_with = "outfile")]
        disassemble: bool,
    },

    /// Poke into memory with value or file
    #[clap(arg_required_else_help = true)]
    Poke {
        /// Destination address, e.g. 4096 (dec) or 0x1000 (hex)
        #[clap(long, short = '@')]
        address: String,
        /// Write bytes from file
        #[clap(long, short = 'f')]
        file: Option<String>,
        /// Byte value to place into memory
        #[clap(value_parser, conflicts_with = "file")]
        value: Option<u8>,
    },

    /// FileHost browser
    #[clap()]
    Filehost {},

    /// Interactive shell environment
    #[clap()]
    Cmd {},
}

#[derive(Parser)]
#[clap(version, about, long_about = None, author = "Copyright (c) 2022 Wombat - Apache/MIT Licensed")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Serial device name, e.g. /dev/cu.usbserial-AQ027F6E
    #[clap(short = 'p', long)]
    pub port: String,

    /// Serial communication speed in bits/s
    #[clap(short = 'b', long, default_value_t = DEFAULT_BAUD_RATE)]
    pub baud: u32,

    /// Verbose output. See more with e.g. RUST_LOG=Trace
    #[clap(long, short = 'v', action)]
    pub verbose: bool,
}
