use std::net::SocketAddrV4;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopAppServices {
    pub redirector_address: SocketAddrV4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub value: u32,
}

#[tarpc::service]
pub trait DesktopAppService {
    /// Ping the service to check if its alive
    async fn ping(value: u32) -> Pong;

    /// Request the services
    async fn services() -> DesktopAppServices;
}
