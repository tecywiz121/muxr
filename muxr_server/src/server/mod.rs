mod acceptor;

use crate::config;
use crate::error::*;

use self::acceptor::Acceptor;

use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
struct Threads {
    acceptor: JoinHandle<Result<()>>,
}

impl Threads {
    fn join_all(self) -> thread::Result<Result<()>> {
        self.acceptor.join()
    }
}

#[derive(Debug)]
enum Status {
    Stopped,
    Stopping,

    Starting,
    Started(Threads),
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

        let threads = Threads {
            acceptor: acceptor_handle,
        };

        state.status = Status::Started(threads);

        Ok(())
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
