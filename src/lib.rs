#[macro_use]
extern crate nom;

use failure::{Error, Fail};

const CHAR_LOOKUP: &'static [u8; 64] =
    b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

#[derive(Fail, Debug)]
#[fail(display = "Error parsing message")]
pub struct ParserError();

#[derive(Debug, PartialEq)]
pub struct ICAOAddress(u8, u8, u8);

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
}

fn decode_callsign(encoded: Vec<u8>) -> String {
    encoded
        .into_iter()
        .map(|b| CHAR_LOOKUP[b as usize] as char)
        .collect::<String>()
}

named!(parse_adsb_message_kind<&[u8], ADSBMessageKind>,
    alt!(
        parse_aircraft_identification
    )
);

named!(parse_aircraft_identification<&[u8], ADSBMessageKind>,
    bits!(
        do_parse!(
            verify!(take_bits!(u8, 5), |v| v >= 1 && v <= 4) >>
            emitter_category: take_bits!(u8, 3) >>
            callsign: map!(many_m_n!(8, 8, take_bits!(u8, 6)), decode_callsign) >>
            (ADSBMessageKind::AircraftIdentification {
                emitter_category,
                callsign,
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
        parse_adsb_message
    )
);

named!(parse_one<&[u8], Message>,
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

pub fn parse_message(message: &[u8]) -> Result<Message, Error> {
    parse_one(message)
        .map(|(_, message)| message)
        .map_err(|_| ParserError().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_aircraft_identification_message() {
        let r = b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98";
        let m = parse_message(r).unwrap();
        assert_eq!(m.downlink_format, 17);
        assert_eq!(
            m.kind,
            MessageKind::ADSBMessage {
                capability: 5,
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
    fn parse_invalid_message() {
        let r = b"\x00";
        assert!(parse_message(r).is_err());
    }
}
