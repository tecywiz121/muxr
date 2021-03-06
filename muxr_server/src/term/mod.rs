use crate::error::*;
use crate::pty;

use futures_util::{future, pin_mut};

use muxr_core::input::{Event, Key};
use muxr_core::state::State;

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt, PollEvented, ReadHalf, WriteHalf};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use vte::{Parser, Perform};

#[derive(Debug)]
pub struct Term {
    state: Arc<Mutex<State>>,
    master: pty::Master,
}

impl Term {
    pub fn new(master: pty::Master, state: Arc<Mutex<State>>) -> Self {
        Term { state, master }
    }

    pub async fn run(self, recv: Receiver<Event>) -> Result<()> {
        let evented = PollEvented::new(self.master)?;
        let (read, write) = tokio::io::split(evented);

        let f0 = Self::write_loop(recv, write);
        let f1 = Self::read_loop(self.state.clone(), read);

        pin_mut!(f0);
        pin_mut!(f1);

        future::try_join(f0, f1).await.map(|_| ())
    }

    async fn write_loop(
        mut recv: Receiver<Event>,
        mut write: WriteHalf<PollEvented<pty::Master>>,
    ) -> Result<()> {
        while let Some(event) = recv.recv().await {
            match event {
                Event::Key(Key::Char(c)) => {
                    let s = c.to_string();

                    write.write_all(s.as_bytes()).await?;
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }

    async fn read_loop(
        state: Arc<Mutex<State>>,
        mut read: ReadHalf<PollEvented<pty::Master>>,
    ) -> Result<()> {
        let mut buf = [0u8; 1024];
        let mut parser = Parser::new();

        loop {
            let len: usize = read.read(&mut buf).await?;
            let bytes = &buf[0..len];

            let mut locked = state.lock().await;
            let mut perform = StatePerform(&mut locked);

            for byte in bytes {
                parser.advance(&mut perform, *byte);
            }
        }
    }
}

#[derive(Debug)]
struct StatePerform<'a>(pub &'a mut State);

impl<'a> Perform for StatePerform<'a> {
    fn print(&mut self, c: char) {
        self.0.print(c)
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            C0::CR => self.0.carriage_return(),
            C0::LF | C0::VT | C0::FF => self.0.linefeed(),
            _ => eprintln!("[UNIMPL] execute({:02x})", byte),
        }
    }

    fn hook(&mut self, a: &[i64], b: &[u8], c: bool) {
        eprintln!("[UNIMPL] hook({:?}, {:?}, {:?})", a, b, c);
    }

    fn put(&mut self, _: u8) {
        unimplemented!()
    }

    fn unhook(&mut self) {
        unimplemented!()
    }

    fn osc_dispatch(&mut self, _: &[&[u8]]) {
        unimplemented!()
    }

    fn csi_dispatch(&mut self, _: &[i64], _: &[u8], _: bool, _: char) {
        unimplemented!()
    }

    fn esc_dispatch(&mut self, _: &[i64], _: &[u8], _: bool, _: u8) {
        unimplemented!()
    }
}

/// C0 set of 7-bit control characters (from ANSI X3.4-1977).
/// Stolen from https://github.com/jwilm/alacritty/blob/96b3d737a8ee1805ec548671a6ba8f219b2c2934/src/ansi.rs
#[allow(non_snake_case)]
#[allow(unused)]
mod C0 {
    /// Null filler, terminal should ignore this character
    pub const NUL: u8 = 0x00;
    /// Start of Header
    pub const SOH: u8 = 0x01;
    /// Start of Text, implied end of header
    pub const STX: u8 = 0x02;
    /// End of Text, causes some terminal to respond with ACK or NAK
    pub const ETX: u8 = 0x03;
    /// End of Transmission
    pub const EOT: u8 = 0x04;
    /// Enquiry, causes terminal to send ANSWER-BACK ID
    pub const ENQ: u8 = 0x05;
    /// Acknowledge, usually sent by terminal in response to ETX
    pub const ACK: u8 = 0x06;
    /// Bell, triggers the bell, buzzer, or beeper on the terminal
    pub const BEL: u8 = 0x07;
    /// Backspace, can be used to define overstruck characters
    pub const BS: u8 = 0x08;
    /// Horizontal Tabulation, move to next predetermined position
    pub const HT: u8 = 0x09;
    /// Linefeed, move to same position on next line (see also NL)
    pub const LF: u8 = 0x0A;
    /// Vertical Tabulation, move to next predetermined line
    pub const VT: u8 = 0x0B;
    /// Form Feed, move to next form or page
    pub const FF: u8 = 0x0C;
    /// Carriage Return, move to first character of current line
    pub const CR: u8 = 0x0D;
    /// Shift Out, switch to G1 (other half of character set)
    pub const SO: u8 = 0x0E;
    /// Shift In, switch to G0 (normal half of character set)
    pub const SI: u8 = 0x0F;
    /// Data Link Escape, interpret next control character specially
    pub const DLE: u8 = 0x10;
    /// (DC1) Terminal is allowed to resume transmitting
    pub const XON: u8 = 0x11;
    /// Device Control 2, causes ASR-33 to activate paper-tape reader
    pub const DC2: u8 = 0x12;
    /// (DC2) Terminal must pause and refrain from transmitting
    pub const XOFF: u8 = 0x13;
    /// Device Control 4, causes ASR-33 to deactivate paper-tape reader
    pub const DC4: u8 = 0x14;
    /// Negative Acknowledge, used sometimes with ETX and ACK
    pub const NAK: u8 = 0x15;
    /// Synchronous Idle, used to maintain timing in Sync communication
    pub const SYN: u8 = 0x16;
    /// End of Transmission block
    pub const ETB: u8 = 0x17;
    /// Cancel (makes VT100 abort current escape sequence if any)
    pub const CAN: u8 = 0x18;
    /// End of Medium
    pub const EM: u8 = 0x19;
    /// Substitute (VT100 uses this to display parity errors)
    pub const SUB: u8 = 0x1A;
    /// Prefix to an escape sequence
    pub const ESC: u8 = 0x1B;
    /// File Separator
    pub const FS: u8 = 0x1C;
    /// Group Separator
    pub const GS: u8 = 0x1D;
    /// Record Separator (sent by VT132 in block-transfer mode)
    pub const RS: u8 = 0x1E;
    /// Unit Separator
    pub const US: u8 = 0x1F;
    /// Delete, should be ignored by terminal
    pub const DEL: u8 = 0x7f;
}

/// C1 set of 8-bit control characters (from ANSI X3.64-1979)
/// Stolen from https://github.com/jwilm/alacritty/blob/96b3d737a8ee1805ec548671a6ba8f219b2c2934/src/ansi.rs
///
/// 0x80 (@), 0x81 (A), 0x82 (B), 0x83 (C) are reserved
/// 0x98 (X), 0x99 (Y) are reserved
/// 0x9a (Z) is 'reserved', but causes DEC terminals to respond with DA codes
#[allow(non_snake_case)]
#[allow(unused)]
mod C1 {
    /// Reserved
    pub const PAD: u8 = 0x80;
    /// Reserved
    pub const HOP: u8 = 0x81;
    /// Reserved
    pub const BPH: u8 = 0x82;
    /// Reserved
    pub const NBH: u8 = 0x83;
    /// Index, moves down one line same column regardless of NL
    pub const IND: u8 = 0x84;
    /// New line, moves done one line and to first column (CR+LF)
    pub const NEL: u8 = 0x85;
    /// Start of Selected Area to be sent to auxiliary output device
    pub const SSA: u8 = 0x86;
    /// End of Selected Area to be sent to auxiliary output device
    pub const ESA: u8 = 0x87;
    /// Horizontal Tabulation Set at current position
    pub const HTS: u8 = 0x88;
    /// Hor Tab Justify, moves string to next tab position
    pub const HTJ: u8 = 0x89;
    /// Vertical Tabulation Set at current line
    pub const VTS: u8 = 0x8A;
    /// Partial Line Down (subscript)
    pub const PLD: u8 = 0x8B;
    /// Partial Line Up (superscript)
    pub const PLU: u8 = 0x8C;
    /// Reverse Index, go up one line, reverse scroll if necessary
    pub const RI: u8 = 0x8D;
    /// Single Shift to G2
    pub const SS2: u8 = 0x8E;
    /// Single Shift to G3 (VT100 uses this for sending PF keys)
    pub const SS3: u8 = 0x8F;
    /// Device Control String, terminated by ST (VT125 enters graphics)
    pub const DCS: u8 = 0x90;
    /// Private Use 1
    pub const PU1: u8 = 0x91;
    /// Private Use 2
    pub const PU2: u8 = 0x92;
    /// Set Transmit State
    pub const STS: u8 = 0x93;
    /// Cancel character, ignore previous character
    pub const CCH: u8 = 0x94;
    /// Message Waiting, turns on an indicator on the terminal
    pub const MW: u8 = 0x95;
    /// Start of Protected Area
    pub const SPA: u8 = 0x96;
    /// End of Protected Area
    pub const EPA: u8 = 0x97;
    /// SOS
    pub const SOS: u8 = 0x98;
    /// SGCI
    pub const SGCI: u8 = 0x99;
    /// DECID - Identify Terminal
    pub const DECID: u8 = 0x9a;
    /// Control Sequence Introducer
    pub const CSI: u8 = 0x9B;
    /// String Terminator (VT125 exits graphics)
    pub const ST: u8 = 0x9C;
    /// Operating System Command (reprograms intelligent terminal)
    pub const OSC: u8 = 0x9D;
    /// Privacy Message (password verification), terminated by ST
    pub const PM: u8 = 0x9E;
    /// Application Program Command (to word processor), term by ST
    pub const APC: u8 = 0x9F;
}

#[cfg(test)]
mod tests {
    mod state {
        use muxr::state::{Col, Row, State};

        #[test]
        fn print_basic() {
            let mut state = State::default();
            state.print('c');

            assert_eq!(state.cursor.position, (Row(0), Col(1)));

            let cell = state.cell(Row(0), Col(0)).unwrap();
            assert_eq!(cell.content, Some('c'));
        }

        #[test]
        fn print_wrap() {
            let mut state = State::with_dimensions(Row(2), Col(1));
            state.print('c');

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

            state.print('c');

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
