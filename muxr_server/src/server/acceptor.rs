use crate::error::*;
use crate::uds::Listener;

use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::{State, Status};

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
        println!("Accepted: {:?}", accepted);
        unimplemented!();
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
