#[macro_use]
extern crate error_chain;
extern crate muxr;
extern crate termion;

mod error;
mod render;

use muxr::state::{State, Color, CellStyle};

use std::io::Write;

use termion::{get_tty, terminal_size};
use termion::raw::IntoRawMode;

fn main() {
    let mut state = State::default();

    let (cols, rows) = terminal_size().unwrap();

    let tty = get_tty().unwrap();

    let mut raw = tty.into_raw_mode().unwrap();

    {
        let cell = state.cell_mut(5, 79).unwrap();
        cell.foreground = Color::new(255, 0, 0);
        cell.content = Some('b');
        cell.style |= CellStyle::BOLD | CellStyle::UNDERSCORE;
    }

    {
        let cell = state.cell_mut(6, 0).unwrap();
        cell.foreground = Color::new(0, 255, 0);
        cell.content = Some('b');
    }

    render::render(&state, &mut raw, rows, cols).unwrap();

    raw.flush().unwrap();
}
