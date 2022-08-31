use clap::{Parser, Subcommand};
/// Matrix Mode Serial Communicator for MEGA65

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Push and run program file
    #[clap(arg_required_else_help = true)]
    Prg {
        /// File to load (*.prg)
        #[clap(value_parser, short = 'f')]
        file: String,
        /// Run after loading
        #[clap(long, short = 'r', action)]
        run: bool,
    },

    /// Get MEGA65 info
    Info {},

    /// Send key presses
    #[clap(arg_required_else_help = true)]
    Type {
        #[clap(value_parser, short = 't')]
        text: String,
    },

    /// List available serial devices
    List {},
    
    /// Reset MEGA65
    Reset {},
}

#[derive(Parser)]
#[clap(version, about, long_about = None, author = "Copyright (c) 2022 Wombat - MIT Licensed")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Serial device name
    #[clap(short = 'p', long)]
    pub port: String,

    /// Baud rate for serial communication
    #[clap(short = 'b', long, default_value_t = 2000000)]
    pub baud: u32,
}
