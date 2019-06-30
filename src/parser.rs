use super::types::*;
use nom::branch::alt;
use nom::combinator::{map, verify};
use nom::multi::many_m_n;
use nom::sequence::tuple;
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

named!(parse_adsb_message_kind<&[u8], ADSBMessageKind>,
    alt!(
        bits!(parse_aircraft_identification) |
        parse_airborne_position |
        parse_airborne_velocity
    )
);

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
    let altitude = (l.rotate_left(4) + r) * q - 1000;
    Ok((input, altitude))
}

fn parse_cpr_parity(input: (&[u8], usize)) -> IResult<(&[u8], usize), Parity> {
    alt((
        map(tag_bits(0b0, 1u8), |_| Parity::Even),
        map(tag_bits(0b1, 1u8), |_| Parity::Odd),
    ))(input)
}

named!(match_tc_airborne_position<(&[u8], usize), u8>, verify!(take_bits!(5u8), |tc| *tc >= 9 && *tc <= 18));
named!(take_1_bit<(&[u8], usize), u8>, take_bits!(1u8));
named!(take_3_bits<(&[u8], usize), u8>, take_bits!(3u8));
named!(parse_coordinate<(&[u8], usize), u32>, take_bits!(17u32));

named!(parse_airborne_position<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            match_tc_airborne_position >>
            take_3_bits >>
            altitude: parse_altitude >>
            take_1_bit >>
            cpr_parity: parse_cpr_parity >>
            cpr_latitude: parse_coordinate >>
            cpr_longitude: parse_coordinate >>
            (ADSBMessageKind::AirbornePosition {
                altitude,
                cpr_frame: CPRFrame {
                    parity: cpr_parity,
                    position: Position {
                        latitude: cpr_latitude.into(),
                        longitude: cpr_longitude.into(),
                    }
                },
            })
        )
    )
);

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

named!(match_tc_airborne_velocity<(&[u8], usize), u8>, verify!(take_bits!(5u8), |tc| *tc == 19));
named!(match_st_airborne_velocity<(&[u8], usize), u8>, verify!(take_bits!(3u8), |st| *st == 1));
named!(parse_velocity<(&[u8], usize), u16>, take_bits!(10u16));
named!(parse_vertical_rate<(&[u8], usize), u16>, take_bits!(9u16));
named!(take_5_bits<(&[u8], usize), u8>, take_bits!(5u8));
named!(take_10_bits<(&[u8], usize), u16>, take_bits!(10u16));

named!(parse_airborne_velocity<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            match_tc_airborne_velocity >>
            match_st_airborne_velocity >>
            take_5_bits >>
            ew_sign: parse_sign >>
            ew_vel: parse_velocity >>
            ns_sign: parse_sign >>
            ns_vel: parse_velocity >>
            vrate_src: parse_vertical_rate_source >>
            vrate_sign: parse_sign >>
            vrate: parse_vertical_rate >>
            take_10_bits >>
            ({
                let v_ew = ((ew_vel as i16 - 1) * ew_sign) as f64;
                let v_ns = ((ns_vel as i16 - 1) * ns_sign) as f64;
                let h = v_ew.atan2(v_ns) * (360.0 / (2.0 * PI));
                let heading = if h < 0.0 { h + 360.0 } else { h };
                ADSBMessageKind::AirborneVelocity {
                    heading,
                    ground_speed: (v_ew.powi(2) + v_ns.powi(2)).sqrt(),
                    vertical_rate: (((vrate - 1) * 64) as i16) * vrate_sign,
                    vertical_rate_source: vrate_src,
                }
            })
        )
    )
);

named!(parse_icao_address<&[u8], ICAOAddress>,
    map!(
        bits!(tuple!(take_bits!(8u8), take_bits!(8u8), take_bits!(8u8))),
        |(a, b, c)| ICAOAddress(a, b, c)
    )
);

named!(parse_adsb_message<&[u8], MessageKind>,
    do_parse!(
        capability: map!(bits!(tuple!(tag_bits!(5u8, 0b10001), take_bits!(3u8))), |(_, ca)| ca) >>
        icao_address: parse_icao_address  >>
        type_code: peek!(bits!(take_bits!(5u8))) >>
        kind: parse_adsb_message_kind >>
        (MessageKind::ADSBMessage {
            capability,
            icao_address,
            type_code,
            kind,
        })
    )
);

named!(parse_message_kind<&[u8], MessageKind>,
    alt!(
        parse_adsb_message |
        value!(MessageKind::Unknown)
    )
);

named!(parse_message<&[u8], Message>,
    do_parse!(
        downlink_format: peek!(bits!(take_bits!(5u8))) >>
        kind: parse_message_kind >>
        // TODO: check CRC
        parse_crc >>
        (Message {
            downlink_format,
            kind,
        })
    )
);

named!(parse_crc<&[u8], u32>, bits!(take_bits!(24u32)));

named!(parse_hex_string<&str, Vec<u8>>,
    many0!(map_res!(take_while_m_n!(2, 2, |d: char| d.is_digit(16)), |d| u8::from_str_radix(d, 16)))
);

named!(parse_avr_frame<&str, Vec<u8>>,
    do_parse!(
        tag!("*") >>
        bytes: parse_hex_string >>
        tag!(";") >>
        (bytes)
    )
);

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
    use super::*;
    const CAPABILITY: u8 = 5;

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
    fn parse_invalid_message() {
        let r = b"\x00";
        assert!(parse_binary(r).is_err());
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
}
