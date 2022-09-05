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

/// Matrix Mode Serial Communicator for MEGA65
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Push and run program file
    #[clap(arg_required_else_help = true)]
    Prg {
        /// File to load or scan (.prg|.d81)
        #[clap(value_parser, short = 'f')]
        file: String,
        /// Reset before loading
        #[clap(long, action)]
        reset: bool,
        /// Run after loading
        #[clap(long, short = 'r', action)]
        run: bool,
    },

    /// Get MEGA65 info
    Info {},

    /// Send key presses
    #[clap(arg_required_else_help = true)]
    Type {
        /// Text to type - use "\r" for return
        #[clap(long, short = 't')]
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
        /// Address to peek into, e.g. 4096 (dec) or 0x4000 (hex)
        #[clap(long, short = '@')]
        address: String,
        /// Number of bytes to retrieve
        #[clap(long = "num", short = 'n', default_value_t = 1)]
        length: usize,
        /// Output to binary file instead of hexdump
        #[clap(long, short = 'o')]
        outfile: Option<String>,
    },

    /// Access FileHost
    #[clap(arg_required_else_help = true)]
    Filehost {
        /// List files
        #[clap(long, short = 'd', action)]
        dir: bool,
    },
}

#[derive(Parser)]
#[clap(version, about, long_about = None, author = "Copyright (c) 2022 Wombat - Apache/MIT Licensed")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Serial device name. Use --port=? to see list of ports.
    #[clap(short = 'p', long)]
    pub port: String,

    /// Baud rate for serial communication
    #[clap(short = 'b', long, default_value_t = 2000000)]
    pub baud: u32,
}
