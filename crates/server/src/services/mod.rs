use std::net::{SocketAddr, SocketAddrV4};

use anyhow::Context;
use tokio::{net::TcpListener, task::AbortHandle};

use crate::{
    services::{blaze::BlazeServer, redirector::RedirectorServer},
    utils::addr::any_localhost,
};

mod blaze;
mod redirector;

/// Handle referencing a service bound to a local socket, contains
/// the address and a handle to abort the task
pub struct ServiceHandle {
    /// Local socket address of the server
    addr: SocketAddrV4,
    /// Handle to abort the server task
    handle: AbortHandle,
}

/// Binds a localhost TCP listener on any available port
pub async fn any_tcp_listener() -> anyhow::Result<(TcpListener, SocketAddrV4)> {
    let listener: TcpListener = TcpListener::bind(any_localhost())
        .await
        .context("failed to bind listener")?;
    let addr = listener
        .local_addr()
        .context("failed to determine local address")?;

    let addr = match addr {
        SocketAddr::V4(addr) => addr,
        SocketAddr::V6(addr) => {
            anyhow::bail!("expected local socket address to be IPv4 but got IPv6: {addr}")
        }
    };

    Ok((listener, addr))
}

pub struct Services {
    blaze: ServiceHandle,
    redirector: ServiceHandle,
    http: ServiceHandle,
}

impl Services {
    pub async fn start() -> anyhow::Result<Services> {
        let blaze = BlazeServer::start()
            .await
            .context("failed to start blaze server")?;
        let redirector = RedirectorServer::start(blaze.addr)
            .await
            .context("failed to start redirector")?;

        Ok(Services {
            blaze,
            redirector,
            http: todo!(),
        })
    }
}
