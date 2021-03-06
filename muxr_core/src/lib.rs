#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate error_chain;
extern crate ndarray;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod input;
pub mod msg;
pub mod state;
