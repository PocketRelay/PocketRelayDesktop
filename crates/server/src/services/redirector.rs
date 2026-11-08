use crate::{
    blaze::{components::redirector, models::redirector::LocalInstanceResponse},
    services::{ServiceHandle, any_tcp_listener},
};
use anyhow::Context;
use blaze_ssl_async::{BlazeAccept, BlazeListener};
use futures::{SinkExt, TryStreamExt};
use pocket_relay_desktop_blaze_networking::{Packet, packet::PacketCodec};
use std::{
    net::{SocketAddr, SocketAddrV4},
    time::Duration,
};
use tokio::{task::AbortHandle, time::timeout};
use tokio_util::codec::Framed;

/// Server for handling blaze connections
pub struct RedirectorServer {
    /// Listener for the redirect connections
    listener: BlazeListener,

    /// Address of the blaze server to redirect to
    blaze_server_address: SocketAddrV4,
}

impl RedirectorServer {
    pub async fn start(blaze_server_address: SocketAddrV4) -> anyhow::Result<ServiceHandle> {
        let (listener, addr) = any_tcp_listener().await?;
        let listener = BlazeListener::from_tokio(listener, Default::default());

        let server = RedirectorServer {
            listener,
            blaze_server_address,
        };

        let handle = tokio::spawn(server.run()).abort_handle();

        Ok(ServiceHandle { addr, handle })
    }

    async fn run(self) {
        loop {
            let client_accept = match self.listener.accept().await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to accept connection");
                    break;
                }
            };

            let blaze_server_address = self.blaze_server_address;

            tokio::spawn(async move {
                tracing::debug!("Redirector connection");
                if let Err(error) = handle(client_accept, blaze_server_address).await {
                    tracing::error!(?error, "error while redirecting");
                }
            });
        }
    }
}

/// Allowed time for a redirect to occur before considering
/// the connection as timed out
const REDIRECT_TIMEOUT: Duration = Duration::from_secs(60);

async fn handle(
    client_accept: BlazeAccept,
    blaze_server_address: SocketAddrV4,
) -> anyhow::Result<()> {
    let (stream, _) = client_accept
        .finish_accept()
        .await
        .context("failed to finish accepting connection")?;
    tracing::debug!("accepted redirect connection");
    let mut framed = Framed::new(stream, PacketCodec::default());

    while let Some(packet) = timeout(REDIRECT_TIMEOUT, framed.try_next())
        .await
        .context("timed out while accepting redirect")?
        .context("failed to read message while accepting redirect")?
    {
        let header = &packet.frame;

        // Respond to unexpected packets with empty responses
        if header.component != redirector::COMPONENT
            || header.command != redirector::GET_SERVER_INSTANCE
        {
            tracing::debug!(
                component = %header.component,
                command = %header.command,
                "redirect got unexpected message",
            );
            framed
                .send(Packet::response_empty(&packet))
                .await
                .context("failed to send redirect response")?;
            continue;
        }

        tracing::debug!("redirector responding");

        framed
            .send(Packet::response(
                &packet,
                LocalInstanceResponse {
                    addr: blaze_server_address,
                },
            ))
            .await
            .context("failed to send redirect response")?;
        break;
    }

    Ok(())
}
