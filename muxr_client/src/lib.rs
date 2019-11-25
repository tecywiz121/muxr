#[macro_use]
extern crate error_chain;

mod error;
mod render;

use muxr_core::state::{CellStyle, Col, Color, Row, State};

use std::io::Write;

use termion::raw::IntoRawMode;
use termion::{get_tty, terminal_size};

pub fn run() {
    let mut state = State::default();

    let (cols, rows) = terminal_size().unwrap();

    let tty = get_tty().unwrap();

    let mut raw = tty.into_raw_mode().unwrap();

    {
        let cell = state.cell_mut(Row(5), Col(79)).unwrap();
        cell.foreground = Color::new(255, 0, 0);
        cell.content = Some('b');
        cell.style |= CellStyle::BOLD | CellStyle::UNDERSCORE;
    }

    {
        let cell = state.cell_mut(Row(6), Col(0)).unwrap();
        cell.foreground = Color::new(0, 255, 0);
        cell.content = Some('c');
    }

    render::render(&state, &mut raw, Row(rows), Col(cols)).unwrap();

    raw.flush().unwrap();
}
