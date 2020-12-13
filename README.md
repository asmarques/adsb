# adsb

[![Crate](https://img.shields.io/crates/v/adsb.svg)](https://crates.io/crates/adsb)
[![Documentation](https://docs.rs/adsb/badge.svg)](https://docs.rs/adsb)
![Build Status](https://github.com/asmarques/adsb/workflows/Continuous%20integration/badge.svg)

A Rust parser for ADS-B/Mode-S messages.

Messages with the following Downlink Formats (DF) are supported:

- **DF 17**: Automatic Dependent Surveillance - Broadcast (ADS-B)
  - **TC 1-4**: Aircraft identification and category
  - **TC 9-18**: Airborne position
  - **TC 19**: Airborne velocity

## Usage

### Parse message in AVR format

```rust
let avr = "*8D4840D6202CC371C32CE0576098;";
let (message, _) = parse_avr(&avr).unwrap();
if let Message {
    kind:
        MessageKind::ADSBMessage {
            kind: ADSBMessageKind::AircraftIdentification { callsign, .. },
            ..
        },
    ..
} = message
{
    println!("callsign: {}", callsign);
}
```

### Parse message in binary format

```rust
let bin = b"\x8D\x40\x62\x1D\x58\xC3\x82\xD6\x90\xC8\xAC\x28\x63\xA7";
let (message, _) = parse_binary(bin).unwrap();
if let Message {
    kind:
        MessageKind::ADSBMessage {
            kind: ADSBMessageKind::AirbornePosition { altitude, .. },
            ..
        },
    ..
} = message
{
    println!("altitude: {}", altitude);
}
```
