use clap::Parser;

mod input;
mod serial;

fn main() {
    pretty_env_logger::init();

    let args = input::Args::parse();

    if args.ports {
        serial::print_ports();
    }

    let mut port = serial::open_port(&args.port, args.baud);

    if args.info {
        serial::hypervisor_info(&mut port);
    }

    if args.reset {
        serial::reset(&mut port).unwrap();
    }

    if args.run {
        serial::type_run(&mut port);
    }
}
