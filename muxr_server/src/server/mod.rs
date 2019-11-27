mod client;

use crate::config;
use crate::error::{Result, ResultExt};

use futures_util::future;
use futures_util::stream::StreamExt;

use muxr_core::input::{Event, Key};
use muxr_core::state::State;

use self::client::Client;

use serde::de::DeserializeOwned;

use std::sync::Arc;
use std::time::Duration;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug)]
struct Inner {
    config: config::Server,
    clients: Mutex<Vec<Client>>,
    state: Arc<Mutex<State>>,
    socket: Mutex<Option<UnixListener>>,
    input_sender: Sender<Event>,
}

#[derive(Debug, Clone)]
pub struct Server(Arc<Inner>);

impl Server {
    pub fn new(
        config: config::Server,
        state: Arc<Mutex<State>>,
        input_sender: Sender<Event>,
    ) -> Result<Self> {
        let socket = UnixListener::bind(&config.socket_path)?;

        let server = Server(Arc::new(Inner {
            clients: Default::default(),
            socket: Mutex::new(Some(socket)),
            input_sender,
            state,
            config,
        }));

        Ok(server)
    }

    pub async fn run(self) -> Result<()> {
        let accept = self.clone().accept_loop();
        let state = self.clone().state_loop();

        future::try_join(accept, state).await.map(|_| ())
    }

    async fn accept_loop(self) -> Result<()> {
        let mut socket = self
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

            tokio::spawn(
                self.clone()
                    .client_read_loop(read, self.0.input_sender.clone()),
            );
            tokio::spawn(self.clone().client_write_loop(write, write_recv));

            self.0.clients.lock().await.push(Client::new(write_send));
        }

        Ok(())
    }

    async fn state_loop(self) -> Result<()> {
        const DELAY: Duration = Duration::from_millis(100);

        let mut interval = tokio::time::interval(DELAY);

        loop {
            interval.tick().await;

            // TODO: Don't copy the byte buffer for each client. Use an Arc, or
            // the bytes crate.

            let bytes = {
                let state = self.0.state.lock().await;
                muxr_core::msg::serialize(&*state)?
            };

            let mut clients = self.0.clients.lock().await;

            let sends = clients.drain(..).map(|mut client| {
                async {
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
    }

    async fn client_read_loop(self, client: ReadHalf<UnixStream>, sender: Sender<Event>) {
        self.client_read(client, sender).await.unwrap();
    }

    async fn client_read(
        self,
        mut client: ReadHalf<UnixStream>,
        mut sender: Sender<Event>,
    ) -> Result<()> {
        loop {
            let event: Event = deserialize_from(&mut client).await?;

            match event {
                Event::Key(Key::Char(k)) => {
                    if let Err(_) = sender.send(Event::Key(Key::Char(k))).await {
                        break;
                    }
                }
                _ => (), // TODO: Handle all other types of characters.
            }
        }

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
            client.write_all(&msg).await?;
        }

        Ok(())
    }
}

async fn deserialize_from<T: DeserializeOwned>(reader: &mut ReadHalf<UnixStream>) -> Result<T> {
    let mut sz_buf = [0u8; std::mem::size_of::<usize>()];
    reader.read_exact(&mut sz_buf).await?;

    let sz = usize::from_ne_bytes(sz_buf) - sz_buf.len();
    let mut msg_buf = vec![0u8; sz];

    reader.read_exact(&mut msg_buf).await?;

    Ok(bincode::deserialize(&msg_buf)?)
}
