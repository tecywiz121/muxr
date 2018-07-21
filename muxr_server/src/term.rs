use bytes::{BufMut, Bytes, BytesMut};

use error::*;

use std::collections::VecDeque;

use tokio_io::codec::{Decoder, Encoder};

use vte::{Parser, Perform};

#[derive(Debug, Clone)]
pub enum ToTerm {
    Bytes(BytesMut),
}

#[derive(Debug, Clone)]
pub enum FromTerm {
    Print(char),
}

#[derive(Debug, Default)]
struct Performer {
    items: VecDeque<FromTerm>,
}

impl Perform for Performer {
    fn print(&mut self, c: char) {
        self.items.push_back(FromTerm::Print(c));
    }

    fn execute(&mut self, v: u8) {
        eprintln!("[UNIMPL] execute({})", v);
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

pub struct VteCodec {
    parser: Parser,
    performer: Performer,
}

impl VteCodec {
    pub fn new() -> Self {
        VteCodec {
            parser: Parser::new(),
            performer: Performer::default(),
        }
    }
}

impl Encoder for VteCodec {
    type Item = ToTerm;
    type Error = Error;

    fn encode(&mut self, item: ToTerm, bytes: &mut BytesMut) -> Result<()> {
        match item {
            ToTerm::Bytes(b) => *bytes = b,
        }

        Ok(())
    }
}

impl Decoder for VteCodec {
    type Item = FromTerm;
    type Error = Error;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Item>> {
        if self.performer.items.is_empty() {
            for byte in bytes.iter() {
                self.parser.advance(&mut self.performer, *byte);
            }

            let len = bytes.len();
            bytes.advance(len);
        }

        Ok(self.performer.items.pop_front())
    }
}
