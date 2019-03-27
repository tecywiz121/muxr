use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    server: Server,
}

#[derive(Debug)]
pub struct Server {
    pub socket_path: PathBuf,
}
