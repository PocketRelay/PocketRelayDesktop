use crate::{
    http::routes,
    services::{ServiceHandle, any_tcp_listener},
};
use axum::{Router, extract::connect_info::IntoMakeServiceWithConnectInfo};
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};

/// Server for handling blaze connections
pub struct HttpServer {
    /// Listener for the http connections
    listener: TcpListener,

    /// Router for the server
    router: IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
}

impl HttpServer {
    pub async fn start() -> anyhow::Result<ServiceHandle> {
        let (listener, addr) = any_tcp_listener().await?;

        let router = routes::router()
            // TODO: Add needed deps
            .into_make_service_with_connect_info::<SocketAddr>();

        let server = HttpServer { listener, router };

        let handle = tokio::spawn(server.run()).abort_handle();

        Ok(ServiceHandle { addr, handle })
    }

    async fn run(self) {
        if let Err(error) = axum::serve(self.listener, self.router)
            .with_graceful_shutdown(async move {
                _ = signal::ctrl_c().await;
            })
            .await
        {
            tracing::error!(?error, "error while running http server");
        }
    }
}
