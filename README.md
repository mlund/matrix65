# Matrix Mode Serial Communicator for MEGA65

This is a CLI tool, `matrix65`, for communicating with the [MEGA65](https://mega65.org/)
8-bit retro computer using a serial connection.
It's akin the `m65` tool provided with the MEGA65 tools
albeit written entirely in **Rust**.

## Features and current status

As of writing, the list of functionality is inferior to `m65`, but
expect more features over time.
Should you be interested in exploring Rust programming,
PR's are welcome.

- [x] World-class memory safety thanks to Rust
- [x] Cross platform serial device detection
- [x] Fancy CLI interface with subcommands
- [x] Logging with e.g. `export RUST_LOG=info`
- [x] Send and run `.prg` file
- [x] Send sequence of key-presses
- [x] Reset MEGA65
- [x] Peek into memory; hexdump
- [x] Load PRG files from CBM disk images (.d81)
- [x] Switch to C64 mode when 0x0801 load address is detected
- [x] MEGA65 Filehost access (experimental)
- [ ] Load at arbitrary address and optionally start with `SYS`
- [ ] Memory dumps in YAML format

## Getting started

The easiest way to get started is to [install Rust](https://www.rust-lang.org/tools/install), followed by:

~~~ bash
cargo install matrix65
~~~

For basic usage, run `matrix65 --help`. Here's another example which transfers the local file `monty.prg`
to the MEGA65; then runs it:

~~~ bash
matrix65 --port /dev/cu.usbserial prg --file monty.prg --run
~~~


## Resources

- MEGA65 Book:
  - Matrix Mode Monitor: section K-13, page 866
  - https://github.com/MEGA65/mega65-user-guide/blob/master/appendix-monitor.tex
- mega65-tools: `src/monitor/monitor.a65`
