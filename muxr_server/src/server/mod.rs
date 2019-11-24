mod acceptor;
mod client;

use crate::config;
use crate::error::*;

use muxr::state::State as MuxrState;

use self::acceptor::Acceptor;

use std::os::unix::net::UnixStream;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
struct Started {
    acceptor: JoinHandle<Result<()>>,

    client_threads: Vec<JoinHandle<Result<()>>>,
    clients: Vec<client::Sender>,
}

impl Started {
    fn join_all(self) -> thread::Result<Result<()>> {
        if let Err(e) = self.acceptor.join()? {
            return Ok(Err(e));
        }

        for client in self.client_threads.into_iter() {
            if let Err(e) = client.join()? {
                return Ok(Err(e));
            }
        }

        Ok(Ok(()))
    }
}

#[derive(Debug)]
enum Status {
    Stopped,
    Stopping,

    Starting,
    Started(Started),
}

impl Status {
    pub fn is_stopped(&self) -> bool {
        match self {
            Status::Stopped => true,
            _ => false,
        }
    }

    pub fn is_stopping(&self) -> bool {
        match self {
            Status::Stopping => true,
            _ => false,
        }
    }

    pub fn is_starting(&self) -> bool {
        match self {
            Status::Starting => true,
            _ => false,
        }
    }

    pub fn is_started(&self) -> bool {
        match self {
            Status::Started(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct State {
    config: config::Server,
    status: Status,
}

impl State {
    fn started_mut(&mut self) -> Option<&mut Started> {
        match self.status {
            Status::Started(ref mut started) => Some(started),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Server {
    state: Arc<RwLock<State>>,
}

impl Server {
    pub fn new(config: config::Server) -> Self {
        let state = State {
            config,
            status: Status::Stopped,
        };

        Server {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let mut acceptor = Acceptor::new(self.state.clone())?;

        let mut state = self.state.write().expect("state lock poisoned");

        if !state.status.is_stopped() {
            bail!("server must be completely stopped before starting");
        }

        state.status = Status::Starting;

        let acceptor_handle = thread::Builder::new()
            .name("acceptor".to_string())
            .spawn(move || acceptor.run())
            .chain_err(|| "unable to start acceptor thread")?;

        let started = Started {
            acceptor: acceptor_handle,
            clients: vec![],
            client_threads: vec![],
        };

        state.status = Status::Started(started);

        Ok(())
    }

    pub fn publish(&mut self, state: &MuxrState) {
        unimplemented!()
    }

    pub fn stop(&mut self) -> Result<()> {
        let threads = {
            let mut state = self.state.write().expect("state lock poisoned");
            let status = std::mem::replace(&mut state.status, Status::Stopping);

            match status {
                Status::Started(t) => t,
                _ => {
                    state.status = status;
                    bail!("server must be started before stopping");
                }
            }
        };

        threads
            .join_all()
            .expect("unable to join all threads")
            .expect("thread returned an error");

        let mut state = self.state.write().expect("state lock poisoned");
        state.status = Status::Stopped;

        Ok(())
    }
}
