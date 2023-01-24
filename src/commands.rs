use crate::filehost;
use crate::textui;
use matrix65::M65Communicator;
use matrix65::io;
use parse_int::parse;

pub fn reset(comm: &mut Box<dyn M65Communicator>, c64: bool) -> Result<(), anyhow::Error> {
    comm.reset()?;
    if c64 {
        comm.go64()?
    };
    Ok(())
}

pub fn peek(
    comm: &mut dyn M65Communicator,
    address: String,
    length: usize,
    outfile: Option<String>,
    disassemble: bool,
) -> Result<(), anyhow::Error> {
    let start_address = parse::<u32>(&address)?;
    let bytes = comm.read_memory(start_address, length)?;
    match outfile {
        Some(name) => io::save_binary(&name, &bytes)?,
        None => {
            if disassemble {
                matrix65::io::disassemble(&bytes, start_address);
            } else {
                matrix65::io::hexdump(&bytes, 8);
            }
        }
    };
    Ok(())
}

pub fn poke(
    file: Option<String>,
    value: Option<u8>,
    address: String,
    comm: &mut dyn M65Communicator,
) -> Result<(), anyhow::Error> {
    let bytes = match file {
        Some(f) => matrix65::io::load_bytes(&f)?,
        None => vec![value.ok_or_else(|| anyhow::Error::msg("VALUE required for poking"))?],
    };
    let parsed_address = parse::<u16>(&address)?;
    if parsed_address.checked_add(bytes.len() as u16 - 1).is_none() {
        // Merely a safety measure. Is this needed?
        return Err(anyhow::Error::msg(
            "poking outside the 16-bit address space is currently unsupported",
        ));
    }
    comm.write_memory(parsed_address, &bytes)?;
    Ok(())
}

pub fn filehost(comm: &mut Box<dyn M65Communicator>) -> Result<(), anyhow::Error> {
    let mut entries: Vec<_> = filehost::get_file_list()?
        .iter()
        .cloned()
        .filter(|item| {
            item.filename.to_lowercase().ends_with(".prg")
                | item.filename.to_lowercase().ends_with(".d81")
        })
        .collect();
    entries.sort_by_key(|i| i.title.clone());
    textui::terminal::start_tui(comm, &entries)?;
    Ok(())
}
