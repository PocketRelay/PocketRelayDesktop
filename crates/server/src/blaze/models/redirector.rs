use std::net::SocketAddrV4;

use tdf::TdfSerialize;

/// Response for redirecting to a local instance
pub struct LocalInstanceResponse {
    pub addr: SocketAddrV4,
}

impl TdfSerialize for LocalInstanceResponse {
    fn serialize<S: tdf::prelude::TdfSerializer>(&self, w: &mut S) {
        w.tag_union_start(b"ADDR", 0x0); /* Server address type */

        // Encode the net address portion
        w.group(b"VALU", |w| {
            w.tag_u32(b"IP", self.addr.ip().to_bits());
            w.tag_u16(b"PORT", self.addr.port());
        });

        w.tag_bool(b"SECU", false);
        w.tag_bool(b"XDNS", false);
    }
}
