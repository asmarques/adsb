//! Decode aircraft positions encoded in Compact Position Reporting (CPR) format.

use crate::types::{CPRFrame, Parity, Position};
use std::cmp;
use std::f64::consts::PI;

const NZ: f64 = 15.0;
const D_LAT_EVEN: f64 = 360.0 / (4.0 * NZ);
const D_LAT_ODD: f64 = 360.0 / (4.0 * NZ - 1.0);
const CPR_MAX: f64 = 131_072.0;

fn cpr_nl(lat: f64) -> u64 {
    let x = 1.0 - (PI / (2.0 * NZ)).cos();
    let y = ((PI / 180.0) * lat).cos().powi(2);
    ((2.0 * PI) / (1.0 - (x / y)).acos()).floor() as u64
}

/// Calculates a globally unambiguous position based on a pair of frames containing position information
/// encoded in CPR format. A position is returned when passed a tuple containing two frames of opposite parity
/// (even and odd). The frames in the tuple should be ordered according to when they were received: the first
/// frame being the oldest frame and the second frame being the latest.
pub fn get_position(cpr_frames: (&CPRFrame, &CPRFrame)) -> Option<Position> {
    let latest_frame = cpr_frames.1;
    let (even_frame, odd_frame) = match cpr_frames {
        (
            even @ CPRFrame {
                parity: Parity::Even,
                ..
            },
            odd @ CPRFrame {
                parity: Parity::Odd,
                ..
            },
        )
        | (
            odd @ CPRFrame {
                parity: Parity::Odd,
                ..
            },
            even @ CPRFrame {
                parity: Parity::Even,
                ..
            },
        ) => (even, odd),
        _ => return None,
    };

    let cpr_lat_even = even_frame.position.latitude / CPR_MAX;
    let cpr_lon_even = even_frame.position.longitude / CPR_MAX;
    let cpr_lat_odd = odd_frame.position.latitude / CPR_MAX;
    let cpr_lon_odd = odd_frame.position.longitude / CPR_MAX;

    let j = (59.0 * cpr_lat_even - 60.0 * cpr_lat_odd + 0.5).floor();

    let mut lat_even = D_LAT_EVEN * (j % 60.0 + cpr_lat_even);
    let mut lat_odd = D_LAT_ODD * (j % 59.0 + cpr_lat_odd);

    if lat_even >= 270.0 {
        lat_even -= 360.0;
    }

    if lat_odd >= 270.0 {
        lat_odd -= 360.0;
    }

    let (lat, mut lon) = if latest_frame == even_frame {
        let ni = cmp::max(cpr_nl(lat_even), 1) as f64;
        let m = (cpr_lon_even * (cpr_nl(lat_even) - 1) as f64
            - cpr_lon_odd * cpr_nl(lat_even) as f64
            + 0.5)
            .floor();
        let lon = (360.0 / ni) * (m % ni + cpr_lon_even);
        let lat = lat_even;
        (lat, lon)
    } else {
        let ni = cmp::max(cpr_nl(lat_odd) - 1, 1) as f64;
        let m = (cpr_lon_even * (cpr_nl(lat_odd) - 1) as f64
            - cpr_lon_odd * cpr_nl(lat_odd) as f64
            + 0.5)
            .floor();
        let lon = (360.0 / ni) * (m % ni + cpr_lon_odd);
        let lat = lat_odd;
        (lat, lon)
    };

    if lon >= 180.0 {
        lon -= 360.0;
    }

    Some(Position {
        latitude: lat,
        longitude: lon,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpr_calculate_position() {
        let odd = CPRFrame {
            position: Position {
                latitude: 74158.0,
                longitude: 50194.0,
            },
            parity: Parity::Odd,
        };

        let even = CPRFrame {
            position: Position {
                latitude: 93000.0,
                longitude: 51372.0,
            },
            parity: Parity::Even,
        };

        let position = get_position((&odd, &even)).unwrap();
        assert_eq!(position.latitude, 52.25720214843750);
        assert_eq!(position.longitude, 3.91937255859375);
    }
}
