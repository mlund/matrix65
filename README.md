[![Crates.io](https://img.shields.io/crates/v/matrix65)](https://crates.io/crates/matrix65)
[![Rust](https://github.com/mlund/matrix65/actions/workflows/rust.yml/badge.svg)](https://github.com/mlund/matrix65/actions/workflows/rust.yml)
[![rust-clippy analyze](https://github.com/mlund/matrix65/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/mlund/matrix65/actions/workflows/rust-clippy.yml)
[![docs.rs](https://img.shields.io/docsrs/matrix65)](https://docs.rs/matrix65/latest/matrix65)

# Matrix Mode Serial Communicator for MEGA65

`matrix65` is a CLI tool, for communicating with the [MEGA65](https://mega65.org/)
8-bit retro computer using a serial connection.
It has a mix of features from `m65`, `mega65_ftp`, and `M65connect` and is written entirely in **Rust**.
Here is a short [demonstration video](https://www.youtube.com/watch?v=dUvXLtUUC-Y).

_Disclaimer: This tool is still under development; use it at your own risk._

## Getting Started

### Installing

The easiest way to get started is to [install the Rust compiler](https://www.rust-lang.org/tools/install), followed by:

~~~ bash
cargo install matrix65
~~~

There are no other dependencies on MacOS and Windows.
On Linux, you may have to install `libudev-dev` (Ubuntu) or `systemd-devel` (Fedora).
To access the serial port, the Linux user may need to be added to the `dialout` group.

### Using

~~~ bash
# transfer prg file and run it (url allowed)
matrix65 --port /dev/myserial prg plasma.prg --run

# text-user-interface (TUI) for the FileHost
matrix65 --port /dev/myserial filehost 

# look for and select PRG file inside disk image (url allowed)
matrix65 --port /dev/myserial prg commando.d81 --run --reset

# type something on the mega65
matrix65 --port /dev/myserial type "dir\n"

# hexdump 16 bytes, starting at $C000
matrix65 --port /dev/myserial peek -@ 0xc000 -n 16
~~~

## Features and current status

As of writing, the list of functionality is inferior to `m65`, but
expect more features over time.
Should you be interested in testing or try some Rust programming,
your involvement is very welcome.

- [x] World-class memory safety thanks to Rust
- [x] Cross platform serial device detection
- [x] Fancy CLI interface with subcommands
- [x] Online Filehost access with TUI (experimental)
- [x] Send and run PRG files
  - [x] Go C64/C65 dependent on detected load address
  - [x] Extract PRG from CBM disk images (.d81)
  - [x] Regular files and URL's are allowed
- [x] Send sequence of key-presses
- [x] Reset MEGA65
- [x] Peek into memory; hexdump, binary file dump
- [x] Poke into memory; single value or from file
- [x] Logging with e.g. `export RUST_LOG=info`
- [x] REPL command interface (experimental)
- [ ] Disassembly
- [ ] Transfer and mount disk images
- [ ] Load at arbitrary address and optionally start with `SYS`
- [ ] Memory dumps in YAML format
- [ ] Bitstream transfer
- [ ] Transfer to SD card

## Motivation

1. Creative fun: I wanted to learn more about Rust and the MEGA65 hardware
2. A Text User Interface (TUI) for the FileHost has an inherent retro-feel
3. Rust
   - is safe and free of undefined behavior
   - compiles to lean, efficient binaries, similar to C
   - has lot's of useful _crates_, e.g. for supporting [CBM disk images](https://crates.io/crates/cbm)
   - has **Cargo** which makes dependency handling easy-peasy
4. In my opinion, higher level language features in Rust are superior to raw C for tasks like
   HTTP requests, JSON parsing, and TUI handling (No offence meant, MEGA65 Team🖖).

## Resources

- MEGA65 Book:
  - Matrix Mode Monitor: section K-13, page 866
  - https://github.com/MEGA65/mega65-user-guide/blob/master/appendix-monitor.tex
- mega65-tools: `src/monitor/monitor.a65`
