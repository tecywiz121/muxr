#![feature(optin_builtin_traits)]

extern crate bytes;
extern crate daemonize;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mio;
extern crate muxr;
extern crate nix;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate vte;

mod error;
mod pty;
mod term;

use error::*;
use pty::CommandTty;
use term::Apply;

use muxr::state::State;

use std::fs::File;
use std::process::Command;

use tokio::prelude::*;

use tokio_codec::Framed;

quick_main!(run);

fn run() -> Result<()> {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = daemonize::Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .start()
        .chain_err(|| "unable to daemonize")?;

    println!("creating pty");

    let (master, slave) = pty::pair().unwrap();

    println!("starting process");
    Command::new("echo")
        .arg("hello world")
        .tty(slave)
        .unwrap()
        .status()
        .unwrap();

    let (writer, reader) = Framed::new(master, term::codec::VteCodec::new()).split();

    let mut state = State::default();

    let app = reader
        .for_each(move |item| {
            println!("TRM: {:?}", item);
            state.apply(item)?;
            Ok(())
        })
        .map_err(|x| println!("ERR: {}", x));

    tokio::run(app);

    Ok(())
}
