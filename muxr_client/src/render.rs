use crate::error::*;

use muxr_core::state::{Cell, CellStyle, Col, Color, Row, State};

use std::io::Write;

use termion as t;

trait WriteDelta {
    fn write_delta<W: Write>(&self, previous: &Self, w: &mut W) -> Result<()>;
}

trait ColorEx {
    fn into_termion(self) -> t::color::Rgb;
}

impl ColorEx for Color {
    fn into_termion(self) -> t::color::Rgb {
        t::color::Rgb(self.r, self.g, self.b)
    }
}

macro_rules! style {
    (
        $self:expr,
        $other:expr,
        $writer:expr,
        $style:ident $(| $style_r:ident)*,
        $enable:ident,
        $disable:ident
    ) => {{
        let style = CellStyle::$style $(| CellStyle::$style_r)*;
        if $self.intersects(style) && !$other.intersects(style) {
            write!($writer, "{}", t::style::$enable)?
        } else if !$self.intersects(style) && $other.intersects(style) {
            write!($writer, "{}", t::style::$disable)?
        }
    }}
}

impl WriteDelta for CellStyle {
    fn write_delta<W: Write>(&self, previous: &Self, w: &mut W) -> Result<()> {
        style!(self, previous, w, BOLD, Bold, NoBold);
        style!(self, previous, w, DIM, Faint, NoFaint);
        style!(self, previous, w, ITALIC, Italic, NoItalic);
        style!(self, previous, w, UNDERSCORE, Underline, NoUnderline);
        style!(self, previous, w, BLINK_FAST | BLINK_SLOW, Blink, NoBlink);
        style!(self, previous, w, REVERSE, Invert, NoInvert);
        style!(self, previous, w, STRIKE, CrossedOut, NoCrossedOut);

        Ok(())
    }
}

impl WriteDelta for Cell {
    fn write_delta<W: Write>(&self, previous: &Self, w: &mut W) -> Result<()> {
        self.style.write_delta(&previous.style, w)?;

        if self.foreground != previous.foreground {
            let fg = self.foreground.into_termion();
            let fg = t::color::Fg(fg);
            write!(w, "{}", fg)?;
        }

        if self.background != previous.background {
            let bg = self.background.into_termion();
            let bg = t::color::Bg(bg);
            write!(w, "{}", bg)?;
        }

        Ok(())
    }
}

pub fn render<W: Write>(state: &State, w: &mut W, rows: Row, cols: Col) -> Result<()> {
    let mut oob = Cell::default();
    oob.style = CellStyle::REVERSE;
    oob.foreground = Color::BLACK;
    oob.background = Color::BLACK;
    oob.content = Some('.');

    let first = Cell::default();
    let mut prev = &first;

    for row in 0..rows.0 {
        write!(w, "{}", t::cursor::Goto(1, row + 1))?;

        for col in 0..cols.0 {
            let cell = state.cell(Row(row), Col(col)).unwrap_or(&oob);
            cell.write_delta(prev, w)?;
            write!(w, "{}", cell.content.unwrap_or(' '))?;
            prev = cell;
        }
    }

    Ok(())
}
