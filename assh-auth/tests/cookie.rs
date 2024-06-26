//! A dummy subservice to test for authentication success.

use assh::{side::Side, Result, Session};
use futures::{AsyncBufRead, AsyncWrite};

const SERVICE_NAME: &str = "dummy-service@assh.rs";

use std::{rc::Rc, sync::atomic::AtomicBool};

#[derive(Debug, Default, Clone)]
pub struct Cookie {
    flag: Rc<AtomicBool>,
}

impl Cookie {
    pub fn is_flagged(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl assh::service::Request for Cookie {
    const SERVICE_NAME: &'static str = SERVICE_NAME;

    type Err = assh::Error;
    type Ok<'s, IO: 's, S: 's> = ();

    async fn on_accept<'s, IO, S>(
        &mut self,
        _: &'s mut Session<IO, S>,
    ) -> Result<Self::Ok<'s, IO, S>, Self::Err>
    where
        IO: AsyncBufRead + AsyncWrite + Unpin,
        S: Side,
    {
        self.flag.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }
}

impl assh::service::Handler for Cookie {
    type Err = assh::Error;
    type Ok<'s, IO: 's, S: 's> = ();

    const SERVICE_NAME: &'static str = SERVICE_NAME;

    async fn on_request<'s, IO, S>(
        &mut self,
        _: &'s mut Session<IO, S>,
    ) -> Result<Self::Ok<'s, IO, S>, Self::Err>
    where
        IO: AsyncBufRead + AsyncWrite + Unpin,
        S: Side,
    {
        self.flag.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }
}
