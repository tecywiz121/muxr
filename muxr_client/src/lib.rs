#[macro_use]
extern crate error_chain;

mod error;
mod io;
mod render;

use crate::error::Result;
use crate::io::{split, ReadHalf, WriteHalf};

use crossbeam_channel::{bounded, select};

use muxr_core::state::{Col, Row, State};

use std::fs::File;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{get_tty, terminal_size};

pub fn run() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let stream = Arc::new(UnixStream::connect("/tmp/muxr.sock")?);

    let tty = get_tty().unwrap();
    let (read, write) = split(tty);

    let raw = write.into_raw_mode()?;

    let (s0, r0) = bounded::<()>(0);
    let (s1, r1) = bounded::<()>(0);

    let stream_clone = stream.clone();
    let running_clone = running.clone();
    let h0 = thread::Builder::new()
        .name("output".into())
        .spawn(move || {
            let result = output_loop(stream_clone, raw, running_clone);
            drop(s0);
            result
        })?;

    let running_clone = running.clone();
    let h1 = thread::Builder::new().name("input".into()).spawn(move || {
        let result = input_loop(stream, read, running_clone);
        drop(s1);
        result
    })?;

    select! {
        recv(r0) -> _ => {
            running.store(false, Ordering::Relaxed);

            h0.join().expect("output thread panicked")?;
            h1.join().expect("input thread panicked")?;
        }
        recv(r1) -> _ => {
            running.store(false, Ordering::Relaxed);

            h1.join().expect("input thread panicked")?;
            h0.join().expect("output thread panicked")?;
        }
    }

    Ok(())
}

fn output_loop(
    stream: Arc<UnixStream>,
    mut raw: RawTerminal<WriteHalf<File>>,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let (cols, rows) = terminal_size().unwrap();

    loop {
        let state: State = muxr_core::msg::deserialize_from(&mut &*stream)?;

        if !running.load(Ordering::Relaxed) {
            break;
        }

        render::render(&state, &mut raw, Row(rows), Col(cols)).unwrap();
        raw.flush().unwrap();
    }

    Ok(())
}

fn input_loop(
    stream: Arc<UnixStream>,
    raw: ReadHalf<File>,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let mut escape = false;

    for event in raw.events() {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let event = event?;

        let escaped = escape;
        escape = false;

        match (escaped, event) {
            (false, Event::Key(Key::Alt('!'))) => {
                escape = true;
            }
            (false, event) => send_event(&*stream, event)?,
            (true, Event::Key(Key::Alt('!'))) => send_event(&*stream, Event::Key(Key::Alt('!')))?,
            (true, Event::Key(Key::Char('q'))) => {
                break;
            }
            (true, Event::Key(_)) => {
                // TODO: Handle unknown escape sequences.
            }
            (true, _) => (),
        }
    }

    Ok(())
}

fn send_event(mut stream: &UnixStream, event: Event) -> Result<()> {
    let key = match event {
        Event::Key(key) => key,
        _ => return Ok(()),
    };

    let mevent = muxr_core::input::Event::Key(match key {
        Key::Backspace => muxr_core::input::Key::Backspace,
        Key::Left => muxr_core::input::Key::Left,
        Key::Right => muxr_core::input::Key::Right,
        Key::Up => muxr_core::input::Key::Up,
        Key::Down => muxr_core::input::Key::Down,
        Key::Home => muxr_core::input::Key::Home,
        Key::End => muxr_core::input::Key::End,
        Key::PageUp => muxr_core::input::Key::PageUp,
        Key::PageDown => muxr_core::input::Key::PageDown,
        Key::Delete => muxr_core::input::Key::Delete,
        Key::Insert => muxr_core::input::Key::Insert,
        Key::F(u) => muxr_core::input::Key::F(u),
        Key::Char(c) => muxr_core::input::Key::Char(c),
        Key::Alt(c) => muxr_core::input::Key::Alt(c),
        Key::Ctrl(c) => muxr_core::input::Key::Ctrl(c),
        Key::Null => muxr_core::input::Key::Null,
        Key::Esc => muxr_core::input::Key::Esc,

        _ => return Ok(()),
    });

    let bytes = muxr_core::msg::serialize(&mevent)?;

    stream.write_all(&bytes)?;

    Ok(())
}
