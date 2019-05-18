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

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ICAOAddress(pub(crate) u8, pub(crate) u8, pub(crate) u8);

impl fmt::Debug for ICAOAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}{:X}{:X}", self.0, self.1, self.2)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CPRFrame {
    pub position: Position,
    pub parity: Parity,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Parity {
    Even,
    Odd,
}

#[derive(Debug, PartialEq, Clone)]
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
    Unknown,
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
    },
    AirborneVelocity {
        heading: f64,
        ground_speed: f64,
        vertical_rate: i16,
        vertical_rate_source: VerticalRateSource,
    },
}
