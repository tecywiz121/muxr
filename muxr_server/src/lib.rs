#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

mod config;
mod error;
mod pty;
mod server;
mod term;

use crate::error::*;
use crate::pty::CommandTty;

use futures_util::pin_mut;
use futures_util::try_future::{self, TryFutureExt};

use muxr_core::state::State;

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::process::Command;
use tokio::sync::Mutex;

pub fn run() -> Result<()> {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    daemonize::Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .start()
        .chain_err(|| "unable to daemonize")?;

    async_run()
}

#[tokio::main]
async fn async_run() -> Result<()> {
    let config = config::Server {
        socket_path: PathBuf::from("/tmp/muxr.sock"),
    };

    let state = Arc::new(Mutex::new(State::default()));

    let server = server::Server::new(config, state.clone())?;

    let mut args = std::env::args_os();

    args.next().chain_err(|| "malformed command line")?;

    let cmd = args.next().chain_err(|| "missing executable path")?;

    let (master, slave) = pty::pair().unwrap();

    let server_run = server.run();

    let cmd_run = Command::new(cmd)
        .args(args)
        .tty(slave)
        .unwrap()
        .status()
        .map_err(Error::from);

    let term_run = term::Term::new(master, state).run();

    pin_mut!(cmd_run);
    pin_mut!(server_run);

    // TODO: Something with exit status

    let part_1 = async {
        match try_future::try_select(cmd_run, server_run).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.factor_first().0),
        }
    };

    pin_mut!(part_1);
    pin_mut!(term_run);

    match try_future::try_select(part_1, term_run).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.factor_first().0),
    }
}
