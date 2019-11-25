#[macro_use]
extern crate error_chain;

mod error;
mod render;

use crate::error::Result;

use muxr_core::state::{Col, Row, State};

use std::io::Write;

use termion::raw::IntoRawMode;
use termion::{get_tty, terminal_size};

use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

pub fn run() -> Result<()> {
    async_run()
}

#[tokio::main]
async fn async_run() -> Result<()> {
    let (cols, rows) = terminal_size().unwrap();

    let tty = get_tty().unwrap();

    let mut raw = tty.into_raw_mode().unwrap();

    let mut stream = UnixStream::connect("/tmp/muxr.sock").await?;

    let mut sz_buf = [0u8; std::mem::size_of::<usize>()];

    while sz_buf.len() == stream.read_exact(&mut sz_buf).await? {
        let sz = usize::from_ne_bytes(sz_buf) - sz_buf.len();
        let mut msg_buf = vec![0u8; sz];

        if msg_buf.len() != stream.read_exact(&mut msg_buf).await? {
            bail!("incomplete read");
        }

        let state: State = bincode::deserialize(&msg_buf)?;
        render::render(&state, &mut raw, Row(rows), Col(cols)).unwrap();
        raw.flush().unwrap();
    }

    Ok(())
}
