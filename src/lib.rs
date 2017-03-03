//! #rust-amqp
//! [![Build Status](https://travis-ci.org/Antti/rust-amq-proto.svg)](https:
//! //travis-ci.org/Antti/rust-amq-proto)
//!
//! AMQ protocol implementation in pure rust.
//!
//! > Note:
//! > The project is still in very early stages of development,
//! > it implements all the protocol parsing, but not all the protocol methods
//! are wrapped/easy to use.
//! > Expect the API to be changed in the future.
//!
//!
//! The methods encoding/decoding code is generated using codegen.rb &
//! amqp-rabbitmq-0.9.1.json spec.
//!
//! To generate a new spec, run:
//!
//! ```sh
//! make
//! ```
//!
//! To build project, use cargo:
//!
//! ```sh
//! cargo build
//! ```
//!
//! To build examples:
//! ```sh
//! cargo test
//! ```

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate byteorder;
extern crate bit_vec;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
#[macro_use] extern crate enum_primitive;

mod framing;
mod table;
mod method;
#[macro_use] mod codegen_macros;
mod error;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub mod protocol;

pub use table::{Table, TableEntry};
pub use method::{Method, };
pub use framing::*;
pub use error::*;
