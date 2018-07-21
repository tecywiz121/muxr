#![feature(optin_builtin_traits)]

extern crate daemonize;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mio;
extern crate muxr;
extern crate nix;

mod error;
mod pty;

use error::*;

use std::process::Command;

use pty::CommandTty;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let daemonize = daemonize::Daemonize::new()
        .start()
        .chain_err(|| "unable to daemonize")?;

    let (master, slave) = pty::pair().unwrap();

    Command::new("bash")
        .arg("-c")
        .arg("sleep 160; echo 'hello world'")
        .tty(slave)
        .unwrap()
        .status()
        .unwrap();

    Ok(())
}
