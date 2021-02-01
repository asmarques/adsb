use super::crc::*;
use super::types::*;
use nom::bits::bits;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::combinator::{map, map_res, peek, verify};
use nom::error::{make_error, ErrorKind};
use nom::multi::{many0, many_m_n};
use nom::sequence::tuple;
use nom::Err;
use nom::IResult;
use nom::{bits::complete::tag as tag_bits, bits::complete::take as take_bits};
use std::f64::consts::PI;

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

fn decode_callsign(encoded: Vec<u8>) -> String {
    encoded
        .into_iter()
        .map(|b| CHAR_LOOKUP[b as usize] as char)
        .collect::<String>()
}

fn parse_aircraft_identification(
    input: (&[u8], usize),
) -> IResult<(&[u8], usize), ADSBMessageKind> {
    let (input, (_, emitter_category, callsign)): (_, (u8, u8, String)) = tuple((
        verify(take_bits(5u8), |tc| *tc >= 1 && *tc <= 4),
        take_bits(3u8),
        map(many_m_n(8, 8, take_bits(6u8)), decode_callsign),
    ))(input)?;
    let message = ADSBMessageKind::AircraftIdentification {
        emitter_category,
        callsign,
    };
    Ok((input, message))
}

fn parse_altitude(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    let (input, (l, q, r)): (_, (u16, u16, u16)) = tuple((
        take_bits(7u8),
        alt((
            map(tag_bits(0b0, 1u8), |_| 100),
            map(tag_bits(0b1, 1u8), |_| 25),
        )),
        take_bits(4u8),
    ))(input)?;
    let altitude = (l.rotate_left(4) + r)
        .checked_mul(q)
        .and_then(|r| r.checked_sub(1000));
    match altitude {
        Some(value) => Ok((input, value)),
        None => Err(Err::Error(make_error(input, ErrorKind::TooLarge))),
    }
}

fn parse_cpr_parity(input: (&[u8], usize)) -> IResult<(&[u8], usize), Parity> {
    alt((
        map(tag_bits(0b0, 1u8), |_| Parity::Even),
        map(tag_bits(0b1, 1u8), |_| Parity::Odd),
    ))(input)
}

fn parse_coordinate(input: (&[u8], usize)) -> IResult<(&[u8], usize), u32> {
    take_bits(17u32)(input)
}

fn parse_airborne_position(input: (&[u8], usize)) -> IResult<(&[u8], usize), ADSBMessageKind> {
    let (input, _): (_, (u8, u8)) = tuple((
        verify(take_bits(5u8), |tc| *tc >= 9 && *tc <= 18),
        take_bits(3u8),
    ))(input)?;

    let (input, (altitude, _)): (_, (u16, u8)) = tuple((parse_altitude, take_bits(1u8)))(input)?;
    let (input, cpr_parity) = parse_cpr_parity(input)?;
    let (input, (cpr_latitude, cpr_longitude)) =
        tuple((parse_coordinate, parse_coordinate))(input)?;

    let message = ADSBMessageKind::AirbornePosition {
        altitude,
        cpr_frame: CPRFrame {
            parity: cpr_parity,
            position: Position {
                latitude: cpr_latitude.into(),
                longitude: cpr_longitude.into(),
            },
        },
    };
    Ok((input, message))
}

fn parse_vertical_rate_source(
    input: (&[u8], usize),
) -> IResult<(&[u8], usize), VerticalRateSource> {
    use VerticalRateSource::*;
    alt((
        map(tag_bits(0b0, 1u8), |_| BarometricPressureAltitude),
        map(tag_bits(0b1, 1u8), |_| GeometricAltitude),
    ))(input)
}

fn parse_sign(input: (&[u8], usize)) -> IResult<(&[u8], usize), i16> {
    alt((
        map(tag_bits(0b0, 1u8), |_| 1),
        map(tag_bits(0b1, 1u8), |_| -1),
    ))(input)
}

fn parse_velocity(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    take_bits(10u16)(input)
}

fn parse_vertical_rate(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    take_bits(9u16)(input)
}

fn parse_airborne_velocity(input: (&[u8], usize)) -> IResult<(&[u8], usize), ADSBMessageKind> {
    let (input, _): (_, (u8, u8, u8)) = tuple((
        verify(take_bits(5u8), |tc| *tc == 19),
        verify(take_bits(3u8), |st| *st == 1),
        take_bits(5u8),
    ))(input)?;

    let (input, (ew_sign, ew_vel)): (_, (i16, u16)) = tuple((parse_sign, parse_velocity))(input)?;
    let (input, (ns_sign, ns_vel)): (_, (i16, u16)) = tuple((parse_sign, parse_velocity))(input)?;
    let (input, (vrate_src, vrate_sign, vrate_value, _)): (_, (VerticalRateSource, i16, u16, u16)) =
        tuple((
            parse_vertical_rate_source,
            parse_sign,
            parse_vertical_rate,
            take_bits(10u16),
        ))(input)?;

    let v_ew = ((ew_vel as i16 - 1) * ew_sign) as f64;
    let v_ns = ((ns_vel as i16 - 1) * ns_sign) as f64;
    let h = v_ew.atan2(v_ns) * (360.0 / (2.0 * PI));
    let heading = if h < 0.0 { h + 360.0 } else { h };

    let vrate = vrate_value
        .checked_sub(1)
        .and_then(|v| v.checked_mul(64))
        .map(|v| (v as i16) * vrate_sign)
        .ok_or_else(|| Err::Error(make_error(input, ErrorKind::TooLarge)))?;

    let message = ADSBMessageKind::AirborneVelocity {
        heading,
        ground_speed: (v_ew.powi(2) + v_ns.powi(2)).sqrt(),
        vertical_rate: vrate,
        vertical_rate_source: vrate_src,
    };
    Ok((input, message))
}

fn parse_icao_address(input: (&[u8], usize)) -> IResult<(&[u8], usize), ICAOAddress> {
    let (input, (a, b, c)): (_, (u8, u8, u8)) =
        tuple((take_bits(8u8), take_bits(8u8), take_bits(8u8)))(input)?;
    let address = ICAOAddress(a, b, c);
    Ok((input, address))
}

fn parse_adsb_message_kind(input: (&[u8], usize)) -> IResult<(&[u8], usize), ADSBMessageKind> {
    alt((
        parse_aircraft_identification,
        parse_airborne_position,
        parse_airborne_velocity,
    ))(input)
}

fn parse_adsb_message(input: (&[u8], usize)) -> IResult<(&[u8], usize), MessageKind> {
    let (input, (_, capability)): (_, (u8, u8)) = tuple((
        alt((
            // If the message comes from a Mode S transponder, it uses DF=17. "Non-transponder-based
            // ADS-B transmitting subsystems and TIS-B transmitting equipment" use DF=18. Note that
            // DF=18 is also sometimes used by systems that generate synthetic messages representing
            // multilaterated aircraft positions.
            tag_bits(0b10001 /* DF=17 */, 5u8),
            tag_bits(0b10010 /* DF=18 */, 5u8),
        )),
        take_bits(3u8),
    ))(input)?;

    let (input, (icao_address, type_code, kind)) = tuple((
        parse_icao_address,
        peek(take_bits(5u8)),
        parse_adsb_message_kind,
    ))(input)?;

    let message = MessageKind::ADSBMessage {
        capability,
        icao_address,
        type_code,
        kind,
    };

    Ok((input, message))
}

fn parse_unknown(input: (&[u8], usize)) -> IResult<(&[u8], usize), MessageKind> {
    Ok((input, MessageKind::Unknown))
}

// In the squawk (identity) field bits are interleaved as follows in
// (message bit 20 to bit 32):
//
// C1-A1-C2-A2-C4-A4-ZERO-B1-D1-B2-D2-B4-D4
//
// So every group of three bits A, B, C, D represent an integer from 0 to 7.
//
// The actual meaning is just 4 octal numbers, but we convert it into a hex
// number tha happens to represent the four octal numbers.
//
// For more info: http://en.wikipedia.org/wiki/Gillham_code

pub fn decode_id_13_field(f: u16) -> u16 {
    let mut hex_gillham = 0;
    if f & 0x1000 != 0 {
        hex_gillham |= 0x0010;
    } // Bit 12 = C1
    if f & 0x0800 != 0 {
        hex_gillham |= 0x1000;
    } // Bit 11 = A1
    if f & 0x0400 != 0 {
        hex_gillham |= 0x0020;
    } // Bit 10 = C2
    if f & 0x0200 != 0 {
        hex_gillham |= 0x2000;
    } // Bit  9 = A2
    if f & 0x0100 != 0 {
        hex_gillham |= 0x0040;
    } // Bit  8 = C4
    if f & 0x0080 != 0 {
        hex_gillham |= 0x4000;
    } // Bit  7 = A4
      //if (ID13Field & 0x0040) {hexGillham |= 0x0800;} // Bit  6 = X  or M
    if f & 0x0020 != 0 {
        hex_gillham |= 0x0100;
    } // Bit  5 = B1
    if f & 0x0010 != 0 {
        hex_gillham |= 0x0001;
    } // Bit  4 = D1 or Q
    if f & 0x0008 != 0 {
        hex_gillham |= 0x0200;
    } // Bit  3 = B2
    if f & 0x0004 != 0 {
        hex_gillham |= 0x0002;
    } // Bit  2 = D2
    if f & 0x0002 != 0 {
        hex_gillham |= 0x0400;
    } // Bit  1 = B4
    if f & 0x0001 != 0 {
        hex_gillham |= 0x0004;
    } // Bit  0 = D4

    hex_gillham
}

fn parse_surveillance_identity(input: (&[u8], usize)) -> IResult<(&[u8], usize), ModeSMessageKind> {
    let (_input, (_df, _flight_status, _downlink_req, _utility_msg, id_code, _parity)): (
        _,
        (u8, u8, u8, u8, u16, u32),
    ) = tuple((
        tag_bits(0b00101, 5u8),
        take_bits(3u8),
        take_bits(5u8),
        take_bits(6u8),
        take_bits(13u8),
        take_bits(24u8),
    ))(input)?;
    let squawk_code = decode_id_13_field(id_code);
    Ok((
        input,
        ModeSMessageKind::SurveillanceIdentity {
            squawk: Squawk::from_u16(squawk_code),
        },
    ))
}

fn parse_mode_s_message_kind(input: (&[u8], usize)) -> IResult<(&[u8], usize), ModeSMessageKind> {
    parse_surveillance_identity(input)
}

fn parse_mode_s_message(input: (&[u8], usize)) -> IResult<(&[u8], usize), MessageKind> {
    let (input, kind) = parse_mode_s_message_kind(input)?;
    let crc = mode_s_crc(input.0, 7)
        .map_err(|_| Err::Error(make_error(input, ErrorKind::LengthValue)))?;
    let icao = (
        (crc & 0xFF0000) >> 16,
        (crc & 0x00FF00) >> 8,
        crc & 0x0000FF,
    );
    let message = MessageKind::ModeSMessage {
        icao_address: ICAOAddress(icao.0 as u8, icao.1 as u8, icao.2 as u8),
        kind,
    };

    Ok((input, message))
}

fn parse_message(input: &[u8]) -> IResult<&[u8], Message> {
    let (input, (downlink_format, kind, _)): (_, (u8, MessageKind, u32)) = bits(tuple((
        peek(take_bits(5u8)),
        alt((parse_mode_s_message, parse_adsb_message, parse_unknown)),
        // TODO: check CRC
        take_bits(24u32),
    )))(input)?;

    let message = Message {
        downlink_format,
        kind,
    };
    Ok((input, message))
}

fn parse_avr_frame(input: &str) -> IResult<&str, Vec<u8>> {
    let (input, _) = tag("*")(input)?;
    let (input, bytes) = many0(map_res(
        take_while_m_n(2, 2, |d: char| d.is_digit(16)),
        |d| u8::from_str_radix(d, 16),
    ))(input)?;
    let (input, _) = tag(";")(input)?;
    Ok((input, bytes))
}

/// Parse message from binary data. If successful, returns a tuple containing the parsed message and a slice
/// of remaining unparsed binary data.
pub fn parse_binary(data: &[u8]) -> Result<(Message, &[u8]), ParserError> {
    let (remaining, message) = parse_message(data)?;
    Ok((message, remaining))
}

/// Parse message from a string with data in AVR format. Each message should start with a `*` and end with a `;`.
/// If successful, returns a tuple containing the parsed message and a slice of remaining unparsed data.
pub fn parse_avr(data: &str) -> Result<(Message, &str), ParserError> {
    let (remaining, frame) = parse_avr_frame(data)?;
    let (_, message) = parse_message(&frame)?;
    Ok((message, remaining))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    const CAPABILITY: u8 = 5;

    #[test]
    fn test_parse_mode_s_0() {
        let r = b"\x28\x00\x1d\x8a\x2d\xa5\xae\x00\x00"; // AC3857 airborne squawking 5670.
        let (_remaining, mm) = parse_mode_s_message_kind((r, 0)).expect("parse error");
        assert_eq!(
            mm,
            ModeSMessageKind::SurveillanceIdentity {
                squawk: Squawk::from_str("5670").unwrap()
            }
        );
    }

    #[test]
    fn test_parse_mode_s_1() {
        let r = b"\x28\x00\x08\x08\xF4\x60\xE0\x00\x00\x00\x00"; // squawk 1200
        let (_remaining, mm) = parse_mode_s_message((r, 0)).expect("parse error");
        assert_eq!(
            mm,
            MessageKind::ModeSMessage {
                icao_address: ICAOAddress(0xA4, 0x04, 0x42),
                kind: ModeSMessageKind::SurveillanceIdentity {
                    squawk: Squawk::from_str("1200").unwrap()
                }
            }
        );
    }

    #[test]
    fn test_parse_mode_s_2() {
        let r = b"\x28\x00\x08\x08\xF4\x60\xE0\x00\x00\x00\x00"; // squawk 1200
        let (_remaining, mm) = parse_mode_s_message((r, 0)).expect("parse error");
        assert_eq!(
            mm,
            MessageKind::ModeSMessage {
                icao_address: ICAOAddress(0xA4, 0x04, 0x42),
                kind: ModeSMessageKind::SurveillanceIdentity {
                    squawk: Squawk::from_str("1200").unwrap()
                }
            }
        );
    }

    #[test]
    fn parse_aircraft_identification_message() {
        let r = b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98";
        let (_, m) = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 17);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: CAPABILITY,
                icao_address: ICAOAddress(0x48, 0x40, 0xD6),
                type_code: 4,
                kind: ADSBMessageKind::AircraftIdentification {
                    emitter_category: 0,
                    callsign: "KLM1023 ".to_string(),
                }
            }
        );
    }

    #[test]
    fn parse_airborne_position_even_message() {
        let r = b"\x8D\x40\x62\x1D\x58\xC3\x82\xD6\x90\xC8\xAC\x28\x63\xA7";
        let (_, m) = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 17);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: CAPABILITY,
                icao_address: ICAOAddress(0x40, 0x62, 0x1D),
                type_code: 11,
                kind: ADSBMessageKind::AirbornePosition {
                    altitude: 38000,
                    cpr_frame: CPRFrame {
                        parity: Parity::Even,
                        position: Position {
                            latitude: 93000.0,
                            longitude: 51372.0,
                        }
                    },
                }
            }
        );
    }

    #[test]
    fn parse_airborne_position_odd_message() {
        let r = b"\x8D\x40\x62\x1D\x58\xC3\x86\x43\x5C\xC4\x12\x69\x2A\xD6";
        let (_, m) = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 17);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: CAPABILITY,
                icao_address: ICAOAddress(0x40, 0x62, 0x1D),
                type_code: 11,
                kind: ADSBMessageKind::AirbornePosition {
                    altitude: 38000,
                    cpr_frame: CPRFrame {
                        parity: Parity::Odd,
                        position: Position {
                            latitude: 74158.0,
                            longitude: 50194.0,
                        }
                    },
                }
            }
        );
    }

    #[test]
    fn parse_airborne_velocity_ground_speed() {
        let r = b"\x8D\x48\x50\x20\x99\x44\x09\x94\x08\x38\x17\x5B\x28\x4F";
        let (_, m) = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 17);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: CAPABILITY,
                icao_address: ICAOAddress(0x48, 0x50, 0x20),
                type_code: 19,
                kind: ADSBMessageKind::AirborneVelocity {
                    heading: 182.8803775528476,
                    ground_speed: 159.20113064925135,
                    vertical_rate: -832,
                    vertical_rate_source: VerticalRateSource::BarometricPressureAltitude,
                }
            }
        );
    }

    #[test]
    fn parse_df18_airborne_position_even_message() {
        // This is a TIS-B message.
        let r = b"\x95\x29\x82\xE5\x68\x1B\x82\xB2\x2B\xB7\xE6\x34\xAE\x96";
        let (_, m) = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 18);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: CAPABILITY,
                icao_address: ICAOAddress(0x29, 0x82, 0xE5),
                type_code: 13,
                kind: ADSBMessageKind::AirbornePosition {
                    altitude: 4400,
                    cpr_frame: CPRFrame {
                        parity: Parity::Even,
                        position: Position {
                            latitude: 88341.0,
                            longitude: 112614.0,
                        }
                    },
                }
            }
        );
    }

    #[test]
    fn parse_single_avr_frame() {
        let r = "*8D4840D6202CC371C32CE0576098;";
        let (_, m) = parse_avr_frame(&r).unwrap();
        assert_eq!(
            m,
            b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98"
        );
    }

    #[test]
    #[allow(unused_must_use)]
    fn parse_invalid_messages() {
        parse_binary(b"\x00");
        parse_binary(b"\x8a\x8f\xff`J\xb4\xc0");
        parse_binary(b"\x8a\xba\x8a#\x99\xff\x04\x00\x00\x00a");
    }
}
