use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Client {
    sender: Sender<Vec<u8>>,
}

impl Client {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self { sender }
    }

    pub async fn send(&mut self, data: Vec<u8>) -> Result<(), SendError> {
        self.sender.send(data).await
    }
}
