use error::*;

use ndarray::Array2;

use std::io::Write;

use termion as t;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Beam,
    Underline,
}

impl Default for CursorStyle {
    fn default() -> Self {
        CursorStyle::Block
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    _p: (),

    pub r: u8,
    pub g: u8,
    pub b: u8,
}

trait WriteDelta {
    fn write_delta<W: Write>(&self, previous: &Self, w: &mut W) -> Result<()>;
}

impl Color {
    pub const BLACK: Self = Color { _p: (), r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Color { _p: (), r: 255, g: 255, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { _p: (), r, g, b }
    }
}

impl From<Color> for t::color::Rgb {
    fn from(o: Color) -> Self {
        t::color::Rgb(o.r, o.g, o.b)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    _p: (),

    pub row: u16,
    pub col: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    _p: (),

    pub position: Position,
    pub color: Color,
    pub style: CursorStyle,
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            _p: (),
            position: Position::default(),
            color: Color::WHITE,
            style: CursorStyle::default(),
            visible: true,
        }
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct CellStyle: u8 {
        const NORMAL        = 0b00000000;
        const BOLD          = 0b00000001;
        const DIM           = 0b00000010;
        const ITALIC        = 0b00000100;
        const UNDERSCORE    = 0b00001000;
        const BLINK_SLOW    = 0b00010000;
        const BLINK_FAST    = 0b00100000;
        const REVERSE       = 0b01000000;
        const STRIKE        = 0b10000000;
    }
}

impl Default for CellStyle {
    fn default() -> Self {
        CellStyle::NORMAL
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    _p: (),

    pub style: CellStyle,
    pub foreground: Color,
    pub background: Color,
    pub content: Option<char>,
}

impl WriteDelta for Cell {
    fn write_delta<W: Write>(&self, previous: &Self, w: &mut W) -> Result<()> {
        self.style.write_delta(&previous.style, w)?;

        if self.foreground != previous.foreground {
            let fg: t::color::Rgb = self.foreground.into();
            let fg = t::color::Fg(fg);
            write!(w, "{}", fg)?;
        }

        if self.background != previous.background {
            let bg: t::color::Rgb = self.background.into();
            let bg = t::color::Bg(bg);
            write!(w, "{}", bg)?;
        }

        Ok(())
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            _p: (),
            style: CellStyle::default(),
            foreground: Color::WHITE,
            background: Color::BLACK,
            content: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    cursor: Cursor,

    cells: Array2<Cell>,
}

impl Default for State {
    fn default() -> Self {
        State {
            cursor: Cursor::default(),
            cells: Array2::default((80, 24)),
        }
    }
}

impl State {
    const OUT_OF_BOUNDS: Cell = Cell {
        _p: (),
        style: CellStyle::REVERSE,
        foreground: Color::BLACK,
        background: Color::BLACK,
        content: Some('.'),
    };


    pub fn cell(&self, row: u16, col: u16) -> &Cell {
        match self.cells.get([col as usize, row as usize]) {
            Some(ref c) => c,
            None => &State::OUT_OF_BOUNDS,
        }
    }

    pub fn cell_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        self.cells.get_mut([col as usize, row as usize])
    }

    pub fn render<W: Write>(&self, w: &mut W, rows: u16, cols: u16) -> Result<()> {
        let first = Cell::default();
        let mut prev = &first;

        for row in 0..rows {
            write!(w, "{}", t::cursor::Goto(1, row + 1))?;

            for col in 0..cols {
                let cell = self.cell(row, col);
                cell.write_delta(prev, w)?;
                write!(w, "{}", cell.content.unwrap_or(' '))?;
                prev = cell;
            }
        }

        Ok(())
    }
}
