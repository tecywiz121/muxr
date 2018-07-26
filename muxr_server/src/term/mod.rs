pub mod codec;

use bytes::BytesMut;

use error::*;

use muxr::state::{Col, Color, CursorStyle, Row, State as MuxrState};

#[derive(Debug, Clone)]
pub enum ToTerm {
    Bytes(BytesMut),
}

#[derive(Debug, Clone)]
pub enum FromTerm {
    // SetTitle
    // SetMouseCursor
    SetCursorStyle(CursorStyle),
    Print(char),
    Goto { row: Row, col: Col },
    GotoRow(Row),
    GotoCol(Col),
    // InsertBlank
    MoveUp(Row),
    MoveDown(Row),
    // IdentifyTerminal
    // DeviceStatus
    MoreForward(Col),
    MoveBackward(Col),
    MoveDownAndReturn(Row),
    MoveUpAndReturn(Row),
    PutTab(i64),
    Backspace,
    CarriageReturn,
    Linefeed,
    // Bell
    // Substitute
    Newline,
    // SetHorizontalTabstop,
    // ScrollUp(u16),
    // ScrollDown(u16),
    // InsertBlankLines(u16),
    // DeleteLines(u16),
    // EraseChars(u16),
    // DeleteChars(u16),
    // MoveBackwardTabs(i64),
    // MoveForwardTabs(i64),
    // SaveCursorPositon,
    // RestoreCursorPosition,
    // ClearLine,
    // ClearScreen,
    // ClearTabs,
    // ResetState,
    // ReverseIndex,
    // TerminalAttribute,
    // SetMode,
    // UnsetMode,
    // SetScrollingRegion,
    // set_keypad_application_mode
    // unset_keypad_application_mode
    // set_active_charset
    // configure_charset
    SetColor(usize, Color),
    ResetColor(usize),
    // SetClipboard
    // Dectest
}

#[derive(Debug, Clone)]
pub struct State(MuxrState);

impl State {
    fn goto_row(&mut self, row: Row) -> Result<()> {
        self.0.cursor.position.0 = row;
        Ok(())
    }

    fn goto_col(&mut self, col: Col) -> Result<()> {
        self.0.cursor.position.1 = col;
        Ok(())
    }

    fn print(&mut self, c: char) -> Result<()> {
        unimplemented!()
    }

    pub fn apply(&mut self, msg: FromTerm) -> Result<()> {
        use term::FromTerm::*;

        match msg {
            GotoRow(row) => self.goto_row(row),
            GotoCol(col) => self.goto_col(col),
            _ => unimplemented!(),
        }
    }
}
