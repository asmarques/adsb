use super::types::*;
use std::f64::consts::PI;
use std::iter::Iterator;

const CHAR_LOOKUP: &'static [u8; 64] =
    b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

fn decode_callsign(encoded: Vec<u8>) -> String {
    encoded
        .into_iter()
        .map(|b| CHAR_LOOKUP[b as usize] as char)
        .collect::<String>()
}

named!(parse_adsb_message_kind<&[u8], ADSBMessageKind>,
    alt!(
        parse_aircraft_identification |
        parse_airborne_position |
        parse_airborne_velocity
    )
);

named!(parse_aircraft_identification<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            verify!(take_bits!(u8, 5), |tc| tc >= 1 && tc <= 4) >>
            emitter_category: take_bits!(u8, 3) >>
            callsign: map!(many_m_n!(8, 8, take_bits!(u8, 6)), decode_callsign) >>
            (ADSBMessageKind::AircraftIdentification {
                emitter_category,
                callsign,
            })
        )
    )
);

named!(parse_altitude<(&[u8], usize), u16>,
    do_parse!(
        l: take_bits!(u16, 7) >>
        q: alt!(
            tag_bits!(u8, 1, 0b0) => {|_| 100 } |
            tag_bits!(u8, 1, 0b1) => {|_| 25 }
        ) >>
        r: take_bits!(u16, 4) >>
        ((l.rotate_left(4) + r) * q - 1000)
    )
);

named!(parse_cpr_frame<(&[u8], usize), CPRFrame>,
    alt!(
        tag_bits!(u8, 1, 0b0) => {|_| CPRFrame::Even } |
        tag_bits!(u8, 1, 0b1) => {|_| CPRFrame::Odd }
    )
);

named!(parse_airborne_position<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            verify!(take_bits!(u8, 5), |tc| tc >= 9 && tc <= 18) >>
            take_bits!(u8, 3) >>
            altitude: parse_altitude >>
            take_bits!(u8, 1) >>
            cpr_frame: parse_cpr_frame  >>
            cpr_latitude: take_bits!(u32, 17) >>
            cpr_longitude: take_bits!(u32, 17) >>
            (ADSBMessageKind::AirbornePosition {
                altitude,
                cpr_frame,
                cpr_latitude,
                cpr_longitude
            })
        )
    )
);

named!(parse_vertical_rate_source<(&[u8], usize), VerticalRateSource>,
    alt!(
        tag_bits!(u8, 1, 0b0) => {|_| VerticalRateSource::BarometricPressureAltitude } |
        tag_bits!(u8, 1, 0b1) => {|_| VerticalRateSource::GeometricAltitude }
    )
);

named!(parse_sign<(&[u8], usize), i16>,
    alt!(
        tag_bits!(u8, 1, 0b0) => {|_| 1 } |
        tag_bits!(u8, 1, 0b1) => {|_| -1 }
    )
);

named!(parse_airborne_velocity<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            verify!(take_bits!(u8, 5), |tc| tc == 19) >>
            verify!(take_bits!(u8, 3), |st| st == 1) >>
            take_bits!(u8, 5) >>
            ew_sign: parse_sign >>
            ew_vel: take_bits!(u16, 10) >>
            ns_sign: parse_sign >>
            ns_vel: take_bits!(u16, 10) >>
            vrate_src: parse_vertical_rate_source >>
            vrate_sign: parse_sign >>
            vrate: take_bits!(u16, 9) >>
            take_bits!(u16, 10) >>
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
        bits!(tuple!(take_bits!(u8, 8), take_bits!(u8, 8), take_bits!(u8, 8))),
        |(a, b, c)| ICAOAddress(a, b, c)
    )
);

named!(parse_adsb_message<&[u8], MessageKind>,
    do_parse!(
        capability: map!(bits!(tuple!(tag_bits!(u8, 5, 0b10001), take_bits!(u8, 3))), |(_, ca)| ca) >>
        icao_address: parse_icao_address  >>
        type_code: peek!(bits!(take_bits!(u8, 5))) >>
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
        downlink_format: peek!(bits!(take_bits!(u8, 5))) >>
        kind: parse_message_kind >>
        // TODO: check CRC
        bits!(take_bits!(u32, 24)) >>
        (Message {
            downlink_format,
            kind,
        })
    )
);

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

pub fn parse_binary(data: &[u8]) -> Result<(Message, &[u8]), ParserError> {
    let (remaining, message) = parse_message(data)?;
    Ok((message, remaining))
}

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
                    cpr_frame: CPRFrame::Even,
                    cpr_latitude: 93000,
                    cpr_longitude: 51372,
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
                    cpr_frame: CPRFrame::Odd,
                    cpr_latitude: 74158,
                    cpr_longitude: 50194,
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
