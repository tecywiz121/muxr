use ndarray::Array2;

#[derive(
    From,
    Into,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Serialize,
    Deserialize,
)]
pub struct Row(pub u16);

impl Row {
    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(
    From,
    Into,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Serialize,
    Deserialize,
)]
pub struct Col(pub u16);

impl Col {
    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}


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

impl Color {
    pub const BLACK: Self = Color {
        _p: (),
        r: 0,
        g: 0,
        b: 0,
    };
    pub const WHITE: Self = Color {
        _p: (),
        r: 255,
        g: 255,
        b: 255,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { _p: (), r, g, b }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    _p: (),

    pub position: (Row, Col),
    pub color: Color,
    pub style: CursorStyle,
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            _p: (),
            position: (Row(0), Col(0)),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    _p: (),

    pub style: CellStyle,
    pub foreground: Color,
    pub background: Color,
    pub content: Option<char>,
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
    cells: Array2<Cell>,
    top: usize,

    pub cursor: Cursor,
}

impl Default for State {
    fn default() -> Self {
        State {
            cursor: Cursor::default(),
            top: 0,
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

    pub fn cell(&self, row: Row, col: Col) -> &Cell {
        match self.cells.get([col.as_usize(), row.as_usize()]) {
            Some(ref c) => c,
            None => &State::OUT_OF_BOUNDS,
        }
    }

    pub fn cell_mut(&mut self, row: Row, col: Col) -> Option<&mut Cell> {
        self.cells.get_mut([col.as_usize(), row.as_usize()])
    }

    pub fn rows(&self) -> Row {
        Row::from(self.cells.dim().1 as u16)
    }

    pub fn columns(&self) -> Col {
        Col::from(self.cells.dim().0 as u16)
    }
}
