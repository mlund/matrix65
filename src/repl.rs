use crate::commands;
use crate::serial;
use matrix65::M65Communicator;
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Provide a state to be passed to each command.
/// Main funtion is to store the serial port
struct Context<'a> {
    pub comm: &'a mut Box<dyn M65Communicator>,
}

pub fn start_repl(comm: &mut Box<dyn M65Communicator>) -> Result<()> {
    let context = Context { comm: comm };
    let mut repl = Repl::new(context)
        .with_name("matrix65")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description(env!("CARGO_PKG_DESCRIPTION"))
        .with_banner("Welcome to matrix65!")
        .with_command(Command::new("reset").about("Reset MEGA65"), reset)
        .with_command(Command::new("go64").about("Go to C64 mode"), go64)
        .with_command(Command::new("stop").about("Halt CPU"), stop)
        .with_command(Command::new("start").about("Resume CPU"), start)
        .with_command(
            Command::new("dasm")
                .about("Disassemble memory (prefix hex values w. 0x....)")
                .arg(Arg::new("address").required(true))
                .arg(Arg::new("length").required(true)),
            peek,
        )
        .with_command(
            Command::new("filehost").about("Start the filehost"),
            filehost,
        );
    repl.run()
}

/// Helper function to convert error type
fn handle_result(result: core::result::Result<(), anyhow::Error>) -> Result<Option<String>> {
    match result {
        Err(err) => Err(reedline_repl_rs::Error::IllegalDefaultError(
            err.to_string(),
        )),
        Ok(()) => Ok(None),
    }
}

/// Wrap peek command
fn peek(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let address = _args.value_of("address").unwrap().to_string();
    let length = _args
        .value_of("length")
        .unwrap_or("1")
        .to_string()
        .parse::<usize>()?;
    let result = commands::peek(context.comm, address, length, None, true);
    handle_result(result)
}

/// Wrap reset command
fn reset(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    handle_result(commands::reset(context.comm, false))
}

/// Wrap go64 command
fn go64(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    handle_result(serial::go64(context.comm))
}

/// Wrap stop cpu command
fn stop(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    handle_result(serial::stop_cpu(context.comm))
}

/// Wrap start cpu command
fn start(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    handle_result(serial::start_cpu(context.comm))
}

/// Wrap filehost command
fn filehost(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    handle_result(commands::filehost(context.comm))
}
