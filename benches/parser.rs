use adsb::parse_binary;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("adsb_aircraft_identification", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98",
            ))
        })
    });
    c.bench_function("adsb_airborne_position_even", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x40\x62\x1D\x58\xC3\x82\xD6\x90\xC8\xAC\x28\x63\xA7",
            ))
        })
    });
    c.bench_function("adsb_airborne_position_odd", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x40\x62\x1D\x58\xC3\x86\x43\x5C\xC4\x12\x69\x2A\xD6",
            ))
        })
    });
    c.bench_function("adsb_airborne_velocity_ground_speed", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x48\x50\x20\x99\x44\x09\x94\x08\x38\x17\x5B\x28\x4F",
            ))
        })
    });
    c.bench_function("mode_s_surveillance_identity", |b| {
        b.iter(|| parse_binary(black_box(b"\x28\x00\x1d\x8a\x2d\xa5\xae")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
