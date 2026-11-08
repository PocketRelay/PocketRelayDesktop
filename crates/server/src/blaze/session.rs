use futures::{TryFutureExt, future::BoxFuture};
use pocket_relay_desktop_blaze_networking::{
    blaze_socket::{BlazeLock, BlazeLockFuture, BlazeRx, BlazeSocketFuture, BlazeTx},
    packet::{Packet, PacketCodec},
    router::BlazeRouter,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Weak,
    task::{Context, Poll},
};
use std::{sync::Arc, task::ready};
use tokio::{net::TcpStream, spawn};
use tokio_util::codec::Framed;
use tracing::{Instrument, Span};

use crate::debug_packet;

pub type SessionLink = Arc<Session>;
pub type WeakSessionLink = Weak<Session>;

pub struct Session {
    /// Handle for sending packets to this session
    tx: BlazeTx,

    /// Data associated with the session
    pub data: SessionData,
}

pub struct SessionData {}

impl Session {
    fn new(tx: BlazeTx, data: SessionData) -> Self {
        Self { tx, data }
    }

    #[tracing::instrument(name = "session", skip_all)]
    pub fn start(io: TcpStream, router: Arc<BlazeRouter<SessionLink>>) {
        // Create blaze socket handler
        let (blaze_future, blaze_rx, blaze_tx) =
            BlazeSocketFuture::new(Framed::new(io, PacketCodec::default()));

        let session_span = Span::current();
        let data = SessionData {};

        let blaze_future = blaze_future
            .inspect_err(|error| tracing::error!(?error, "error running blaze socket future"))
            .inspect_ok(|_| tracing::debug!("session blaze future completed"))
            .instrument(session_span.clone());

        // Create session handler
        let session = Arc::new(Self::new(blaze_tx, data));
        let session_future = session
            .run(blaze_rx, router)
            .instrument(session_span.clone());

        spawn(blaze_future);
        spawn(session_future);

        tracing::debug!("session started");
    }

    // Extra layer to provide borrowed references of session and router so
    // that the session future lifetimes can be qualified
    async fn run(self: Arc<Self>, rx: BlazeRx, router: Arc<BlazeRouter<SessionLink>>) {
        SessionFuture::new(rx, &self, &router).await;
        tracing::debug!("session future complete");
    }

    pub fn notify(&self, packet: Packet) {
        self.tx.notify(packet);
    }
}

/// Future for processing a session
///
/// Produces [Poll::Ready] when the session has completed / closed
struct SessionFuture<'a> {
    /// Receiver for packets to handle
    rx: BlazeRx,
    /// The session this link is for
    session: &'a SessionLink,
    /// The router to use
    router: &'a BlazeRouter<SessionLink>,
    /// State of the future
    state: SessionFutureState<'a>,
}

/// Session future reading state
enum SessionFutureState<'a> {
    /// Waiting for inbound packet
    Accept,
    /// Waiting to acquire write handling lock
    Acquire {
        /// Future for the locking guard
        lock_future: BlazeLockFuture,
        /// The packet that was read
        packet: Option<Packet>,
    },
    /// Future for a handler is being polled
    Handle {
        /// Access to the sender for sending the response
        tx: BlazeLock,
        /// Handle future
        future: BoxFuture<'a, Packet>,
    },
}

impl SessionFuture<'_> {
    pub fn new<'a>(
        rx: BlazeRx,
        session: &'a Arc<Session>,
        router: &'a BlazeRouter<SessionLink>,
    ) -> SessionFuture<'a> {
        SessionFuture {
            router,
            rx,
            session,
            state: SessionFutureState::Accept,
        }
    }
}

impl Drop for SessionFuture<'_> {
    fn drop(&mut self) {
        // Clear session data, speeds up process of ending the session
        // prevents session data being accessed while shutting down
        tracing::debug!("session auth cleared, session dropped");
    }
}

impl Future for SessionFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            // Poll checking if the connection has timed-out
            // if this.session.data.poll_keep_alive_dead(cx) {
            //     return Poll::Ready(());
            // }

            match &mut this.state {
                SessionFutureState::Accept => {
                    let packet = match ready!(this.rx.poll_recv(cx)) {
                        Some(value) => value,
                        None => {
                            // Read half of the socket has terminated, nothing left to handle
                            return Poll::Ready(());
                        }
                    };

                    // Acquire a write lock future (Reserve our space for sending the response)
                    let lock_future = Box::pin(this.session.tx.acquire_tx());

                    this.state = SessionFutureState::Acquire {
                        lock_future,
                        packet: Some(packet),
                    }
                }
                SessionFutureState::Acquire {
                    lock_future,
                    packet,
                } => {
                    let guard = ready!(Pin::new(lock_future).poll(cx));
                    let packet = packet
                        .take()
                        .expect("Unexpected acquire state without packet");

                    debug_packet!(&packet, direction = "request", "receiving packet");

                    let future = this.router.handle(this.session.clone(), packet);

                    // Move onto a handling state
                    this.state = SessionFutureState::Handle { tx: guard, future };
                }
                SessionFutureState::Handle { tx, future } => {
                    // Poll the handler until completion
                    let response = ready!(Pin::new(future).poll(cx));

                    debug_packet!(&response, direction = "response", "sending packet");

                    // Send the response to the writer
                    if tx.send(response).is_err() {
                        // Write half has closed, cease reading
                        return Poll::Ready(());
                    }

                    // Reset back to the reading state
                    this.state = SessionFutureState::Accept;
                }
            }
        }
    }
}
