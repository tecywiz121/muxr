#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Key(Key),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,

    #[serde(skip)]
    #[doc(hidden)]
    __NonExhaustive,
}
