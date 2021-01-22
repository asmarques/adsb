//! Decode aircraft positions encoded in Compact Position Reporting (CPR) format.

use crate::types::{CPRFrame, Parity, Position};
use std::cmp;

const NZ: f64 = 15.0;
const D_LAT_EVEN: f64 = 360.0 / (4.0 * NZ);
const D_LAT_ODD: f64 = 360.0 / (4.0 * NZ - 1.0);
const CPR_MAX: f64 = 131_072.0;

// The NL function uses the precomputed table from 1090-WP-9-14
// This code is translated from https://github.com/wiedehopf/readsb/blob/dev/cpr.c

pub fn cpr_nl(lat: f64) -> u64 {
    let mut lat = lat;
    if lat < 0.0 {
        // Table is symmetric about the equator
        lat = -lat;
    }
    if lat < 29.91135686 {
        if lat < 10.47047130 {
            return 59;
        }
        if lat < 14.82817437 {
            return 58;
        }
        if lat < 18.18626357 {
            return 57;
        }
        if lat < 21.02939493 {
            return 56;
        }
        if lat < 23.54504487 {
            return 55;
        }
        if lat < 25.82924707 {
            return 54;
        }
        if lat < 27.93898710 {
            return 53;
        }
        // < 29.91135686
        return 52;
    }
    if lat < 44.19454951 {
        if lat < 31.77209708 {
            return 51;
        }
        if lat < 33.53993436 {
            return 50;
        }
        if lat < 35.22899598 {
            return 49;
        }
        if lat < 36.85025108 {
            return 48;
        }
        if lat < 38.41241892 {
            return 47;
        }
        if lat < 39.92256684 {
            return 46;
        }
        if lat < 41.38651832 {
            return 45;
        }
        if lat < 42.80914012 {
            return 44;
        }
        // < 44.19454951
        return 43;
    }
    if lat < 59.95459277 {
        if lat < 45.54626723 {
            return 42;
        }
        if lat < 46.86733252 {
            return 41;
        }
        if lat < 48.16039128 {
            return 40;
        }
        if lat < 49.42776439 {
            return 39;
        }
        if lat < 50.67150166 {
            return 38;
        }
        if lat < 51.89342469 {
            return 37;
        }
        if lat < 53.09516153 {
            return 36;
        }
        if lat < 54.27817472 {
            return 35;
        }
        if lat < 55.44378444 {
            return 34;
        }
        if lat < 56.59318756 {
            return 33;
        }
        if lat < 57.72747354 {
            return 32;
        }
        if lat < 58.84763776 {
            return 31;
        }
        // < 59.95459277
        return 30;
    }
    if lat < 61.04917774 {
        return 29;
    }
    if lat < 62.13216659 {
        return 28;
    }
    if lat < 63.20427479 {
        return 27;
    }
    if lat < 64.26616523 {
        return 26;
    }
    if lat < 65.31845310 {
        return 25;
    }
    if lat < 66.36171008 {
        return 24;
    }
    if lat < 67.39646774 {
        return 23;
    }
    if lat < 68.42322022 {
        return 22;
    }
    if lat < 69.44242631 {
        return 21;
    }
    if lat < 70.45451075 {
        return 20;
    }
    if lat < 71.45986473 {
        return 19;
    }
    if lat < 72.45884545 {
        return 18;
    }
    if lat < 73.45177442 {
        return 17;
    }
    if lat < 74.43893416 {
        return 16;
    }
    if lat < 75.42056257 {
        return 15;
    }
    if lat < 76.39684391 {
        return 14;
    }
    if lat < 77.36789461 {
        return 13;
    }
    if lat < 78.33374083 {
        return 12;
    }
    if lat < 79.29428225 {
        return 11;
    }
    if lat < 80.24923213 {
        return 10;
    }
    if lat < 81.19801349 {
        return 9;
    }
    if lat < 82.13956981 {
        return 8;
    }
    if lat < 83.07199445 {
        return 7;
    }
    if lat < 83.99173563 {
        return 6;
    }
    if lat < 84.89166191 {
        return 5;
    }
    if lat < 85.75541621 {
        return 4;
    }
    if lat < 86.53536998 {
        return 3;
    }
    if lat < 87.00000000 {
        return 2;
    }
    return 1;
}

/// Calculates a globally unambiguous position based on a pair of frames containing position information
/// encoded in CPR format. A position is returned when passed a tuple containing two frames of opposite parity
/// (even and odd). The frames in the tuple should be ordered according to when they were received: the first
/// frame being the oldest frame and the second frame being the latest.
pub fn get_position(cpr_frames: (&CPRFrame, &CPRFrame)) -> Option<Position> {
    let latest_frame = cpr_frames.1;
    let (even_frame, odd_frame) = match cpr_frames {
        (
            even
            @ CPRFrame {
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
            even
            @ CPRFrame {
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

    let lat = if latest_frame == even_frame {
        lat_even
    } else {
        lat_odd
    };

    let (lat, lon) = get_lat_lon(lat, cpr_lon_even, cpr_lon_odd, &latest_frame.parity);

    Some(Position {
        latitude: lat,
        longitude: lon,
    })
}

fn get_lat_lon(lat: f64, cpr_lon_even: f64, cpr_lon_odd: f64, parity: &Parity) -> (f64, f64) {
    let (p, c) = if parity == &Parity::Even {
        (0, cpr_lon_even)
    } else {
        (1, cpr_lon_odd)
    };
    let ni = cmp::max(cpr_nl(lat) - p, 1) as f64;
    let m =
        (cpr_lon_even * (cpr_nl(lat) - 1) as f64 - cpr_lon_odd * cpr_nl(lat) as f64 + 0.5).floor();
    let mut lon = (360.0 / ni) * (m % ni + c);
    if lon >= 180.0 {
        lon -= 360.0;
    }
    (lat, lon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_cpr_nl() {
        assert_eq!(cpr_nl(89.9), 1);
        assert_eq!(cpr_nl(-89.9), 1);
        assert_eq!(cpr_nl(86.9), 2);
    }

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
        assert_approx_eq!(position.latitude, 52.2572021484375);
        assert_approx_eq!(position.longitude, 3.91937255859375);
    }

    #[test]
    fn cpr_calculate_position_high_lat() {
        let even = CPRFrame {
            position: Position {
                latitude: 108011.0,
                longitude: 110088.0,
            },
            parity: Parity::Even,
        };
        let odd = CPRFrame {
            position: Position {
                latitude: 75050.0,
                longitude: 36777.0,
            },
            parity: Parity::Odd,
        };
        let position = get_position((&even, &odd)).unwrap();
        assert_approx_eq!(position.latitude, 88.91747426178496);
        assert_approx_eq!(position.longitude, 101.01104736328125);
    }
}
