use clap::Parser;

/// Matrix Mode Serial Communicator for MEGA65
#[derive(Parser)]
#[clap(version, about, long_about = None, author = "Copyright (c) 2022 Wombat - MIT Licensed")]
pub struct Args {
    /// Reset MEGA65
    #[clap(long, action)]
    pub reset: bool,

    /// Serial device name
    #[clap(short = 'p', long)]
    pub port: String,

    /// List available serial devices
    #[clap(long, action)]
    pub ports: bool,

    /// Baud rate for serial communication
    #[clap(short = 'b', long, default_value_t = 2000000)]
    pub baud: u32,

    /// Fetch MEGA65 info
    #[clap(long, action)]
    pub info: bool,
}
