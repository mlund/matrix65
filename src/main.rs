use clap::Parser;

mod input;
mod serial;

// /dev/cu.usbserial-AQ027F6E

fn main() {
    pretty_env_logger::init();

    let args = input::Args::parse();

    let mut port = serial::open_port(&args.port, args.baud);

    match args.command {
        input::Commands::List {} => {
            serial::print_ports();
        }
        input::Commands::Info {} => {
            serial::hypervisor_info(&mut port);
        }
        input::Commands::Reset {} => {
            serial::reset(&mut port).unwrap();
        }
        input::Commands::Type { text } => {
            serial::type_text(&mut port, &text.as_str());
        }
        input::Commands::Prg { file, run } => {
            if run {
                serial::type_text(&mut port, "run\r");
            }
        }
    }
}
