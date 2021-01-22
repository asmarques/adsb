use adsb::{cpr::cpr_nl, parse_binary};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("aircraft_identification", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x48\x40\xD6\x20\x2C\xC3\x71\xC3\x2C\xE0\x57\x60\x98",
            ))
        })
    });
    c.bench_function("airborne_position_even", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x40\x62\x1D\x58\xC3\x82\xD6\x90\xC8\xAC\x28\x63\xA7",
            ))
        })
    });
    c.bench_function("airborne_position_odd", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x40\x62\x1D\x58\xC3\x86\x43\x5C\xC4\x12\x69\x2A\xD6",
            ))
        })
    });
    c.bench_function("airborne_velocity_ground_speed", |b| {
        b.iter(|| {
            parse_binary(black_box(
                b"\x8D\x48\x50\x20\x99\x44\x09\x94\x08\x38\x17\x5B\x28\x4F",
            ))
        })
    });
    c.bench_function("cpr_nl_high_lat", |b| b.iter(|| cpr_nl(black_box(89.0))));
    c.bench_function("cpr_nl_low_lat", |b| b.iter(|| cpr_nl(black_box(0.0))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
