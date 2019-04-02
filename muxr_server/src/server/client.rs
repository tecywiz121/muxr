use crate::error::*;

use std::time::Duration;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::os::unix::net::UnixStream;

use super::{Status, State};

#[derive(Debug)]
pub struct Sender {
    socket: UnixStream,
    state: Arc<RwLock<State>>,
}

#[derive(Debug)]
pub struct Receiver {
    socket: UnixStream,
    state: Arc<RwLock<State>>,
}

pub fn pair(state: Arc<RwLock<State>>, socket: UnixStream) -> Result<(Sender, Receiver)> {
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    let recv_sock = socket.try_clone()?;

    let sender = Sender { socket, state: state.clone() };

    let receiver = Receiver { socket: recv_sock, state };

    Ok((sender, receiver))
}

impl Receiver {
    fn is_running(&self) -> bool {
        match self.state.read().expect("state lock poisoned").status {
            Status::Starting | Status::Started(_) => true,
            _ => false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while self.is_running() {
            // TODO: Do something with the socket data.
            let mut buf = [0u8];
            self.socket.read_exact(&mut buf)?;
        }

        Ok(())
    }
}
