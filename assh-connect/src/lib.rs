#![doc = concat!(
    "[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), ")](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")",
    " ",
    "[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), ")](https://docs.rs/", env!("CARGO_PKG_NAME"), ")",
    " ",
    "![license](https://img.shields.io/crates/l/", env!("CARGO_PKG_NAME"), ")"
)]
#![doc = ""]
#![doc = env!("CARGO_PKG_DESCRIPTION")]
//!

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]
#![forbid(unsafe_code)]

const SERVICE_NAME: &str = "ssh-connection";

const MAXIMUM_PACKET_SIZE: u32 = 32768; // 32KiB
const INITIAL_WINDOW_SIZE: u32 = 64 * MAXIMUM_PACKET_SIZE;
const WINDOW_ADJUST_THRESHOLD: u32 = INITIAL_WINDOW_SIZE / 2;

pub mod channel;
pub mod connect;

mod error;
pub use error::{Error, Result};

use assh::{service, side::Side, Session};
use futures::{AsyncBufRead, AsyncWrite};

/// An implementation of [`service::Handler`] and [`service::Request`] that yields a [`connect::Connect`] instance.
pub struct Service;

impl service::Handler for Service {
    type Err = assh::Error;
    type Ok<'s, IO: 's, S: 's> = connect::Connect<'s, IO, S>;

    const SERVICE_NAME: &'static str = SERVICE_NAME;

    async fn on_request<'s, IO, S>(
        &mut self,
        session: &'s mut Session<IO, S>,
    ) -> Result<Self::Ok<'s, IO, S>, Self::Err>
    where
        IO: AsyncBufRead + AsyncWrite + Unpin,
        S: Side,
    {
        Ok(connect::Connect::new(session))
    }
}

impl service::Request for Service {
    type Err = assh::Error;
    type Ok<'s, IO: 's, S: 's> = connect::Connect<'s, IO, S>;

    const SERVICE_NAME: &'static str = SERVICE_NAME;

    async fn on_accept<'s, IO, S>(
        &mut self,
        session: &'s mut Session<IO, S>,
    ) -> Result<Self::Ok<'s, IO, S>, Self::Err>
    where
        IO: AsyncBufRead + AsyncWrite + Unpin,
        S: Side,
    {
        Ok(connect::Connect::new(session))
    }
}
