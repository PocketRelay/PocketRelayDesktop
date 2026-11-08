use crate::{
    blaze::session::{Session, SessionLink},
    services::{ServiceHandle, any_tcp_listener},
};
use pocket_relay_desktop_blaze_networking::{BlazeRouter, BlazeRouterBuilder};
use std::sync::Arc;
use tokio::net::TcpListener;

/// Server for handling blaze connections
pub struct BlazeServer {
    listener: TcpListener,
    router: Arc<BlazeRouter<SessionLink>>,
}

impl BlazeServer {
    pub async fn start() -> anyhow::Result<ServiceHandle> {
        let (listener, addr) = any_tcp_listener().await?;

        let router = BlazeRouterBuilder::new()
            // TODO: Actually setup routes
            .build();

        let server = BlazeServer { listener, router };
        let handle = tokio::spawn(server.run()).abort_handle();

        Ok(ServiceHandle { addr, handle })
    }

    async fn run(self) {
        loop {
            let client_stream = match self.listener.accept().await {
                Ok((value, _)) => value,
                Err(error) => {
                    tracing::error!(?error, "failed to accept connection");
                    break;
                }
            };

            Session::start(client_stream, self.router.clone());
        }
    }
}
