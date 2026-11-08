use crate::packet::Packet;
use crate::router::IntoPacketResponse;
pub type ServerResult<T> = Result<T, BlazeError>;

#[derive(Debug, Clone)]
#[repr(u16)]
#[allow(unused)]
pub enum GlobalError {
    Cancelled = 0x4009,
    Disconnected = 0x4006,
    DuplicateLogin = 0x4007,
    AuthorizationRequired = 0x4008,
    Timeout = 0x4005,
    ComponentNotFound = 0x4002,
    CommandNotFound = 0x4003,
    AuthenticationRequired = 0x4004,
    System = 0x4001,
}

/// Response type for some blaze error code
pub struct BlazeError(pub u16);

impl From<GlobalError> for BlazeError {
    fn from(value: GlobalError) -> Self {
        BlazeError(value as u16)
    }
}

pub trait CodedBlazeError {
    fn error_code(self) -> u16;
}

impl<T: CodedBlazeError> From<T> for BlazeError {
    fn from(value: T) -> Self {
        BlazeError(value.error_code())
    }
}

impl IntoPacketResponse for BlazeError {
    fn into_response(self, req: &Packet) -> Packet {
        Packet::error_empty(req, self.0)
    }
}
