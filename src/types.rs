use std::convert::From;
use std::error::Error;
use std::fmt;

/// Error type used to convey parsing errors.
#[derive(Debug)]
pub struct ParserError(String);

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl<T> From<nom::Err<T>> for ParserError
where
    nom::Err<T>: std::fmt::Debug,
{
    fn from(error: nom::Err<T>) -> Self {
        ParserError(format!("{:?}", error))
    }
}

/// Unique 24-bit ICAO address assigned to an aircraft upon national registration.
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ICAOAddress(pub(crate) u8, pub(crate) u8, pub(crate) u8);

impl fmt::Debug for ICAOAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}{:X}{:X}", self.0, self.1, self.2)
    }
}

/// Horizontal coordinates in the geographic coordinate system.
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

/// Aircraft position is broadcast as a set of alternating odd and even frames
/// which encode position information using Compact Position Reporting (CPR).
#[derive(Debug, PartialEq, Clone)]
pub struct CPRFrame {
    /// Aircraft position in CPR format
    pub position: Position,
    /// Frame parity
    pub parity: Parity,
}

/// Frame parity.
#[derive(Debug, PartialEq, Clone)]
pub enum Parity {
    Even,
    Odd,
}

/// Source for vertical rate information.
#[derive(Debug, PartialEq, Clone)]
pub enum VerticalRateSource {
    /// Barometric pressure altitude change rate
    BarometricPressureAltitude,
    /// Geometric altitude change rate
    GeometricAltitude,
}

/// ADS-B/Mode-S message.
#[derive(Debug)]
pub struct Message {
    /// Downlink Format (DF)
    pub downlink_format: u8,
    /// Kind of message
    pub kind: MessageKind,
}

/// Kind of ADS-B/Mode-S message.
#[derive(Debug, PartialEq)]
pub enum MessageKind {
    /// ADSB message (DF 17)
    ADSBMessage {
        capability: u8,
        icao_address: ICAOAddress,
        type_code: u8,
        kind: ADSBMessageKind,
    },
    /// Unsupported message
    Unknown,
}

/// Kind of ADSB message.
#[derive(Debug, PartialEq)]
pub enum ADSBMessageKind {
    /// Aicraft identification and category message (TC 1-4)
    AircraftIdentification {
        /// Emitter category used to determine the type of aircraft
        emitter_category: u8,
        /// Aircraft callsign
        callsign: String,
    },
    /// Airborne position message (TC 9-18)
    AirbornePosition {
        /// Altitude in feet
        altitude: u16,
        /// Odd or even frame encoding position information in CPR format
        cpr_frame: CPRFrame,
    },
    /// Airborne velocity message (TC 19)
    AirborneVelocity {
        /// Heading in degrees
        heading: f64,
        /// Ground speed in knots
        ground_speed: f64,
        /// Vertical rate in feet per minute, positive values indicate an aircraft is climbing and
        /// negative values indicate it is descending
        vertical_rate: i16,
        /// Source for vertical rate information
        vertical_rate_source: VerticalRateSource,
    },
}
