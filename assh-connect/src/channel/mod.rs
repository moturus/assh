//! Definition of the [`Channel`] struct that provides isolated I/O on SSH channels.

use std::sync::{atomic::AtomicU32, Arc};

use futures::{AsyncRead, AsyncWrite};
use ssh_packet::connect;

use crate::{Error, Result};

#[doc(no_inline)]
pub use connect::{ChannelExtendedDataType, ChannelRequestContext};

mod io;

mod msg;
pub(super) use msg::Msg;

// TODO: Enable channels read muxing, with RwLock<HashSet<Option<connect::ChannelExtendedDataType>>>

/// A response to a channel request.
#[derive(Debug, PartialEq, Eq)]
pub enum Response {
    /// The request succeeded.
    Success,

    /// The request failed.
    Failure,
}

/// A reference to an opened channel in the session.
#[derive(Debug)]
pub struct Channel {
    remote_id: u32,

    local_window_size: AtomicU32,
    remote_window_size: Arc<AtomicU32>,
    remote_maximum_packet_size: u32,

    sender: flume::Sender<Msg>,
    receiver: flume::Receiver<Msg>,
}

impl Channel {
    pub(super) fn new(
        remote_id: u32,
        local_window_size: u32,
        remote_window_size: Arc<AtomicU32>,
        remote_maximum_packet_size: u32,
        sender: flume::Sender<Msg>,
    ) -> (Self, flume::Sender<Msg>) {
        let (tx, rx) = flume::unbounded();

        (
            Self {
                remote_id,
                local_window_size: local_window_size.into(),
                remote_window_size,
                remote_maximum_packet_size,
                receiver: rx,
                sender,
            },
            tx,
        )
    }

    /// Tells whether the channel has been closed by us or the peer.
    pub fn is_closed(&self) -> bool {
        self.receiver.is_disconnected()
    }

    /// Make a reader for current channel's _data_ stream.
    ///
    /// # Caveats
    ///
    /// Even though the interface allows having multiple _readers_,
    /// polling for a reader will discard other data types
    /// and polling concurrently for more than one reader may cause data integrity issues.
    #[must_use]
    pub fn as_reader(&self) -> impl AsyncRead + '_ {
        io::Read::new(self, None)
    }

    /// Make a reader for current channel's _extended data_ stream.
    ///
    /// # Caveats
    ///
    /// Even though the interface allows having multiple _readers_,
    /// polling for a reader will discard other data types
    /// and polling concurrently for more than one reader may cause data integrity issues.
    #[must_use]
    pub fn as_reader_ext(&self, ext: ChannelExtendedDataType) -> impl AsyncRead + '_ {
        io::Read::new(self, Some(ext))
    }

    /// Make a writer for current channel's _data_ stream.
    #[must_use]
    pub fn as_writer(&self) -> impl AsyncWrite + '_ {
        io::Write::new(self, None)
    }

    /// Make a writer for current channel's _extended data_ stream.
    #[must_use]
    pub fn as_writer_ext(&self, ext: ChannelExtendedDataType) -> impl AsyncWrite + '_ {
        io::Write::new(self, Some(ext))
    }

    /// Send a request on the current channel.
    pub async fn request(&self, context: ChannelRequestContext) -> Result<Response> {
        self.sender
            .send_async(Msg::Request(connect::ChannelRequest {
                recipient_channel: self.remote_id,
                want_reply: true.into(),
                context,
            }))
            .await
            .map_err(|_| Error::ChannelClosed)?;

        match self
            .receiver
            .recv_async()
            .await
            .map_err(|_| Error::ChannelClosed)?
        {
            Msg::Success(_) => Ok(Response::Success),
            Msg::Failure(_) => Ok(Response::Failure),
            _ => Err(assh::Error::UnexpectedMessage.into()),
        }
    }

    /// Receive and handle a request on the current channel.
    pub async fn on_request(
        &self,
        mut handler: impl FnMut(ChannelRequestContext) -> Response,
    ) -> Result<Response> {
        match self
            .receiver
            .recv_async()
            .await
            .map_err(|_| Error::ChannelClosed)?
        {
            Msg::Request(request) => {
                let response = handler(request.context);

                if *request.want_reply {
                    match response {
                        Response::Success => {
                            self.sender
                                .send_async(Msg::Success(connect::ChannelSuccess {
                                    recipient_channel: self.remote_id,
                                }))
                                .await
                                .map_err(|_| Error::ChannelClosed)?;
                        }
                        Response::Failure => {
                            self.sender
                                .send_async(Msg::Failure(connect::ChannelFailure {
                                    recipient_channel: self.remote_id,
                                }))
                                .await
                                .map_err(|_| Error::ChannelClosed)?;
                        }
                    }
                }

                Ok(response)
            }
            _ => Err(assh::Error::UnexpectedMessage.into()),
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        tracing::debug!("Closing channel %{}", self.remote_id);

        if let Err(err) = self.sender.send(Msg::Close(connect::ChannelClose {
            recipient_channel: self.remote_id,
        })) {
            tracing::error!(
                "Unable to report channel %{} closed to the peer: {err}",
                self.remote_id
            );
        }
    }
}
