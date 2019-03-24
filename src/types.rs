use failure::Fail;
use std::convert::From;
use std::fmt;

#[derive(Fail, Debug)]
#[fail(display = "Error parsing message")]
pub struct ParserError();

impl<T> From<nom::Err<T>> for ParserError {
    fn from(_error: nom::Err<T>) -> Self {
        // TODO: add error context
        ParserError()
    }
}

#[derive(PartialEq)]
pub struct ICAOAddress(pub(crate) u8, pub(crate) u8, pub(crate) u8);

impl fmt::Debug for ICAOAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}{:X}{:X}", self.0, self.1, self.2)
    }
}

#[derive(Debug, PartialEq)]
pub enum CPRFrame {
    Odd,
    Even,
}

#[derive(Debug, PartialEq)]
pub enum VerticalRateSource {
    BarometricPressureAltitude,
    GeometricAltitude,
}

#[derive(Debug)]
pub struct Message {
    pub downlink_format: u8,
    pub kind: MessageKind,
}

#[derive(Debug, PartialEq)]
pub enum MessageKind {
    ADSBMessage {
        capability: u8,
        icao_address: ICAOAddress,
        type_code: u8,
        kind: ADSBMessageKind,
    },
}

#[derive(Debug, PartialEq)]
pub enum ADSBMessageKind {
    AircraftIdentification {
        emitter_category: u8,
        callsign: String,
    },
    AirbornePosition {
        altitude: u16,
        cpr_frame: CPRFrame,
        cpr_latitude: u32,
        cpr_longitude: u32,
    },
    AirborneVelocity {
        heading: f64,
        ground_speed: f64,
        vertical_rate: i16,
        vertical_rate_source: VerticalRateSource,
    },
}
