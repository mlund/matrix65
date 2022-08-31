use clap::Parser;

/// Matrix Mode Serial Communicator for MEGA65
#[derive(Parser)]
#[clap(version, about, long_about = None, author = "Copyright (c) 2022 Wombat - MIT Licensed")]
pub struct Args {
    /// Reset MEGA65
    #[clap(long, action)]
    pub reset: bool,

    /// Serial device name
    #[clap(short = 'p', long, required_unless_present("ports"))]
    pub port: String,

    /// Send text string (use "\r" for return)
    #[clap(long, short = 't')]
    pub text: Option<String>,

    /// Run after loading
    #[clap(long, short = 'r', action)]
    pub run: bool,

    /// Load program into memory
    #[clap(long)]
    pub prg: Option<String>,

    /// List available serial devices
    #[clap(long, action, conflicts_with("port"))]
    pub ports: bool,

    /// Baud rate for serial communication
    #[clap(short = 'b', long, default_value_t = 2000000)]
    pub baud: u32,

    /// Fetch MEGA65 info
    #[clap(long, action)]
    pub info: bool,
}
