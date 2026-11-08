use std::fmt::Debug;

use bytes::Bytes;
use tdf::{TdfDeserializer, TdfStringifier};

#[macro_export]
macro_rules! debug_packet {
    ($packet:expr, $minified:expr$(, $($arg:tt)*)?) => {{
        let header = &$packet.frame;
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(
                component = format!("{:#06x}", header.component),
                command = format!("{:#06x}", header.command),
                seq = %header.seq,
                contents = ?$crate::utils::debug_packet::DebugTdfStringify(&$packet.contents),
                $($($arg)*)?
            );
        }
    }};
}

pub struct DebugTdfStringify<'a>(pub &'a Bytes);

impl<'a> Debug for DebugTdfStringify<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return f.write_str("None");
        }

        let r = TdfDeserializer::new(self.0);
        let mut out = String::new();
        let mut s = TdfStringifier::new(r, &mut out);
        let _ = s.stringify();
        f.write_str(&out)
    }
}
