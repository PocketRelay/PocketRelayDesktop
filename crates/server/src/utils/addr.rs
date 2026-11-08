use std::net::{Ipv4Addr, SocketAddrV4};

pub fn any_localhost() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)
}
