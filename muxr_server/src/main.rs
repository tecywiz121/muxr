extern crate bytes;
extern crate daemonize;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mio;
extern crate muxr;
extern crate nix;
extern crate vte;

mod config;
mod error;
mod pty;
mod server;
mod term;
mod uds;

use error::*;
use pty::CommandTty;
use term::StatePerform;

use muxr::state::State;

use std::io;
use std::thread::JoinHandle;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, Arc};

fn parse_args() -> Result<Command> {
    let mut args = std::env::args_os();
    args.next().expect("not enough arguments");

    let exec = args.next().chain_err(|| "missing argument - executable")?;

    let mut cmd = Command::new(exec);
    cmd.args(args);

    Ok(cmd)
}

quick_main!(run);

fn run() -> Result<()> {
    let cmd = parse_args()?;

    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    /*
    let daemonize = daemonize::Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .start()
        .chain_err(|| "unable to daemonize")?;
    */

    let config = config::Server {
        socket_path: PathBuf::from("/tmp/muxr.sock"),
    };

    let mut server = server::Server::new(config);

    server.start()?;

    let state = Arc::new(Mutex::new(State::default()));

    spawn_execute(cmd, state.clone()).join().unwrap();

    server.stop()?;

    println!("{:?}", state.lock().unwrap());

    Ok(())
}

fn distribute(state: Arc<Mutex<State>>) {
    loop {

    }
}

fn spawn_execute(cmd: Command, state: Arc<Mutex<State>>) -> JoinHandle<()> {
    std::thread::spawn(move || execute(cmd, state))
}

fn execute(mut cmd: Command, state: Arc<Mutex<State>>) -> () {
    println!("creating pty");

    let mut parser = vte::Parser::new();

    let (mut master, slave) = pty::pair().unwrap();

    println!("starting process");
    let mut child = cmd
        .tty(slave)
        .unwrap()
        .spawn()
        .unwrap();

    loop {
        let mut buf = [0u8; 1];

        let err = match master.read_exact(&mut buf) {
            Ok(_) => {
                println!("Read: {}", buf[0]);

                let mut write = state.lock().unwrap();
                let mut perf = StatePerform(&mut write);

                parser.advance(&mut perf, buf[0]);

                continue;
            }
            Err(e) => e,
        };

        match err.kind() {
            io::ErrorKind::WouldBlock => {
                if child.try_wait().unwrap().is_some() {
                    println!("Exiting");
                    break;
                }

                // TODO: Do IO properly.
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            other => panic!(other),
        }

    }
}
