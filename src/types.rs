use std::convert::From;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Error type used to convey parsing errors.
#[derive(Debug)]
pub struct ParserError(String);

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParserError {}

impl<T> From<nom::Err<T>> for ParserError
where
    nom::Err<T>: std::fmt::Debug,
{
    fn from(error: nom::Err<T>) -> Self {
        ParserError(format!("{:?}", error))
    }
}

/// Unique 24-bit ICAO address assigned to an aircraft upon national registration.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct ICAOAddress(pub(crate) u8, pub(crate) u8, pub(crate) u8);

impl fmt::Display for ICAOAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

/// 16 bit transponder squawk code.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Squawk(pub(crate) u8, pub(crate) u8);

impl From<u16> for Squawk {
    fn from(value: u16) -> Self {
        Squawk(((value & 0xFF00) >> 8) as u8, (value & 0x00FF) as u8)
    }
}

impl FromStr for Squawk {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(u16::from_str_radix(value, 16)?.into())
    }
}

impl fmt::Display for Squawk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}{:02X}", self.0, self.1)
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
#[derive(Debug, PartialEq)]
pub struct Message {
    /// Downlink Format (DF)
    pub downlink_format: u8,
    /// Kind of message
    pub kind: MessageKind,
}

/// Kind of ADS-B/Mode-S message.
#[derive(Debug, PartialEq, Clone)]
pub enum MessageKind {
    /// ADSB message (DF 17)
    ADSBMessage {
        capability: u8,
        icao_address: ICAOAddress,
        type_code: u8,
        kind: ADSBMessageKind,
    },
    ModeSMessage {
        icao_address: ICAOAddress,
        kind: ModeSMessageKind,
    },
    /// Unsupported message
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModeSMessageKind {
    // DF=5
    SurveillanceIdentity { squawk: Squawk },
}

/// Kind of ADSB message.
#[derive(Debug, PartialEq, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squawk() {
        assert_eq!(Squawk::from(22128), Squawk(86, 112));
        assert_eq!(Squawk::from_str("5670").unwrap(), Squawk(86, 112));
        assert_eq!(format!("{}", Squawk::from_str("5670").unwrap()), "5670");
        // 4608 1200
        assert_eq!(Squawk::from(4608), Squawk(18, 0));
        assert_eq!(Squawk::from_str("1200").unwrap(), Squawk(18, 0));
        assert_eq!(format!("{}", Squawk::from_str("1200").unwrap()), "1200");
    }
}
