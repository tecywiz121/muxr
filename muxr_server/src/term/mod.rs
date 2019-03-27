pub mod codec;

use bytes::BytesMut;

use crate::error::*;

use muxr::state::{Col, Color, CursorStyle, Row, State};

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

mod sealed {
    use super::*;

    pub trait StateEx {
        fn goto_row(&mut self, row: Row) -> Result<()>;
        fn goto_col(&mut self, row: Col) -> Result<()>;
        fn print(&mut self, c: char) -> Result<()>;
        fn carriage_return(&mut self) -> Result<()>;
        fn linefeed(&mut self) -> Result<()>;
    }
}

use self::sealed::StateEx;

pub trait Apply {
    fn apply(&mut self, msg: FromTerm) -> Result<()>;
}

impl<T> Apply for T
where
    T: StateEx,
{
    fn apply(&mut self, msg: FromTerm) -> Result<()> {
        use self::FromTerm::*;

        match msg {
            GotoRow(row) => self.goto_row(row),
            GotoCol(col) => self.goto_col(col),
            Print(c) => self.print(c),
            CarriageReturn => self.carriage_return(),
            Linefeed => self.linefeed(),
            _ => unimplemented!(),
        }
    }
}

impl StateEx for State {
    fn goto_row(&mut self, row: Row) -> Result<()> {
        self.cursor.position.0 = row;
        Ok(())
    }

    fn goto_col(&mut self, col: Col) -> Result<()> {
        self.cursor.position.1 = col;
        Ok(())
    }

    fn print(&mut self, c: char) -> Result<()> {
        let (row, col) = self.cursor.position;

        {
            let cell = self.cell_mut(row, col);

            if let Some(cell) = cell {
                cell.content = Some(c);
            }
        }

        if col >= self.columns() - Col(1) {
            self.cursor.position.0 += Row(1);
            self.cursor.position.1 = Col(0);

            if self.cursor.position.0 >= self.rows() {
                self.cursor.position.0 = self.rows() - Row(1);
                self.scroll_down(Row(1));
            }
        } else {
            self.cursor.position.1 += Col(1);
        }

        Ok(())
    }

    fn carriage_return(&mut self) -> Result<()> {
        self.cursor.position.1 = Col(0);
        Ok(())
    }

    fn linefeed(&mut self) -> Result<()> {
        self.cursor.position.0 += Row(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod state {
        use super::super::StateEx;

        use muxr::state::{Col, Row, State};

        #[test]
        fn print_basic() {
            let mut state = State::default();
            state.print('c').unwrap();

            assert_eq!(state.cursor.position, (Row(0), Col(1)));

            let cell = state.cell(Row(0), Col(0)).unwrap();
            assert_eq!(cell.content, Some('c'));
        }

        #[test]
        fn print_wrap() {
            let mut state = State::with_dimensions(Row(2), Col(1));
            state.print('c').unwrap();

            assert_eq!(state.cursor.position, (Row(1), Col(0)));

            let cell = state.cell(Row(0), Col(0)).unwrap();
            assert_eq!(cell.content, Some('c'));
        }

        #[test]
        fn print_scroll() {
            let mut state = State::with_dimensions(Row(3), Col(1));

            state.cell_mut(Row(0), Col(0)).unwrap().content = Some('a');
            state.cell_mut(Row(1), Col(0)).unwrap().content = Some('b');

            state.cursor.position = (Row(2), Col(0));

            state.print('c').unwrap();

            assert_eq!(state.cursor.position, (Row(2), Col(0)));

            let cell = state.cell(Row(0), Col(0)).unwrap();
            assert_eq!(cell.content, Some('b'));

            let cell = state.cell(Row(1), Col(0)).unwrap();
            assert_eq!(cell.content, Some('c'));

            let cell = state.cell(Row(2), Col(0)).unwrap();
            assert_eq!(cell.content, None);
        }
    }
}
