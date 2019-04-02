use crate::error::*;
use crate::uds::Listener;

use std::os::unix::net::UnixStream;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::thread;

use super::{State, Status};
use super::client;

#[derive(Debug)]
pub struct Acceptor {
    state: Arc<RwLock<State>>,
    socket: Listener,
}

impl Acceptor {
    pub fn new(state: Arc<RwLock<State>>) -> Result<Self> {
        let socket = {
            let guard = state.read().expect("state lock poisoned");
            Listener::bind(&guard.config.socket_path).chain_err(|| "unable to bind unix socket")?
        };

        Ok(Self { state, socket })
    }

    fn is_running(&self) -> bool {
        match self.state.read().expect("state lock poisoned").status {
            Status::Starting | Status::Started(_) => true,
            _ => false,
        }
    }

    fn accept(&self, accepted: UnixStream) -> Result<()> {
        let fd = accepted.as_raw_fd();
        let (sender, mut receiver) = client::pair(self.state.clone(), accepted)?;

        let mut state = self.state.write().expect("state lock poisoned");

        let started = state.started_mut().chain_err(|| "not started")?;

        started.clients.push(sender);

        let thread = thread::Builder::new()
            .name(format!("client-{}", fd))
            .spawn(move || receiver.run())
            .chain_err(|| "unable to start client thread")?;

        started.client_threads.push(thread);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.is_running() {
            let sock = self.socket.accept(Duration::from_secs(1))?;
            if let Some(accepted) = sock {
                self.accept(accepted)?;
            }
        }

        Ok(())
    }
}
