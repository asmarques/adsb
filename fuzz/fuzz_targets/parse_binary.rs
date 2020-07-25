#![no_main]
use adsb;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = adsb::parse_binary(data);
});
