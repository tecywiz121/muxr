mod client;

use bincode;

use crate::config;
use crate::error::{Result, ResultExt};

use futures_util::stream::StreamExt;
use futures_util::{future, try_future};

use muxr_core::state::State;

use self::client::Client;

use std::sync::Arc;
use std::time::Duration;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;
use tokio::timer::Interval;

use tokio_io::split::{ReadHalf, WriteHalf};

#[derive(Debug)]
struct Inner {
    config: config::Server,
    clients: Mutex<Vec<Client>>,
    state: Arc<Mutex<State>>,
    socket: Mutex<Option<UnixListener>>,
}

#[derive(Debug, Clone)]
pub struct Server(Arc<Inner>);

impl Server {
    pub fn new(config: config::Server, state: Arc<Mutex<State>>) -> Result<Self> {
        let socket = UnixListener::bind(&config.socket_path)?;

        let server = Server(Arc::new(Inner {
            clients: Default::default(),
            socket: Mutex::new(Some(socket)),
            state,
            config,
        }));

        Ok(server)
    }

    pub async fn run(self) -> Result<()> {
        let accept = self.clone().accept_loop();
        let state = self.clone().state_loop();

        try_future::try_join(accept, state).await.map(|_| ())
    }

    async fn accept_loop(self) -> Result<()> {
        let socket = self
            .0
            .socket
            .lock()
            .await
            .take()
            .chain_err(|| "server can only be started once")?;

        let mut incoming = socket.incoming();

        while let Some(item) = incoming.next().await {
            let client = match item {
                Ok(c) => c,
                Err(e) => {
                    // TODO: Better logging?
                    eprintln!("accept_loop error: {:?}", e);
                    continue;
                }
            };

            eprintln!("Accepted: {:?}", client);

            let (read, write) = io::split(client);

            let (write_send, write_recv) = channel(1);

            tokio::spawn(self.clone().client_read_loop(read));
            tokio::spawn(self.clone().client_write_loop(write, write_recv));

            self.0.clients.lock().await.push(Client::new(write_send));
        }

        Ok(())
    }

    async fn state_loop(self) -> Result<()> {
        const DELAY: Duration = Duration::from_millis(100);

        let mut interval = Interval::new_interval(DELAY);

        while let Some(_) = interval.next().await {
            // TODO: Don't copy the byte buffer for each client. Use an Arc, or
            // the bytes crate.

            let bytes = {
                const USZ_LEN: usize = std::mem::size_of::<usize>();

                let mut buffer = vec![0u8; USZ_LEN];

                let state = self.0.state.lock().await;
                bincode::serialize_into(&mut buffer, &*state)?;

                let len = buffer.len().to_ne_bytes();
                buffer[0..USZ_LEN].copy_from_slice(&len);

                buffer
            };

            let mut clients = self.0.clients.lock().await;

            let sends = clients.drain(..).map(|mut client| {
                async {
                    eprintln!("sending...");

                    match client.send(bytes.clone()).await {
                        Ok(_) => Some(client),
                        Err(e) => {
                            eprintln!("state_loop error: {:?}", e);
                            None
                        }
                    }
                }
            });

            *clients = future::join_all(sends)
                .await
                .into_iter()
                .filter_map(|x| x)
                .collect();
        }

        Ok(())
    }

    async fn client_read_loop(self, client: ReadHalf<UnixStream>) {
        self.client_read(client).await.unwrap();
    }

    async fn client_read(self, mut client: ReadHalf<UnixStream>) -> Result<()> {
        let mut data = Vec::new();

        // TODO: Do something with the incoming data
        client.read_to_end(&mut data).await?;

        Ok(())
    }

    async fn client_write_loop(self, client: WriteHalf<UnixStream>, recv: Receiver<Vec<u8>>) {
        self.client_write(client, recv).await.unwrap();
    }

    async fn client_write(
        self,
        mut client: WriteHalf<UnixStream>,
        mut recv: Receiver<Vec<u8>>,
    ) -> Result<()> {
        while let Some(msg) = recv.recv().await {
            eprintln!("Writing {} bytes", msg.len());
            client.write_all(&msg).await?;
        }

        Ok(())
    }
}
