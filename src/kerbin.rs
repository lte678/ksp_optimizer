use std::{iter::zip, sync::atomic::AtomicBool};

pub const ATMOSPHERE: &[(f32, f32, f32)] = &[
    (0.0    , 1.000, 1.225),
    (2500.0 , 0.681, 0.898),
    (5000.0 , 0.450, 0.642),
    (7500.0 , 0.287, 0.446),
    (10000.0, 0.177, 0.288),
    (15000.0, 0.066, 0.108),
    (20000.0, 0.025, 0.040),
    (25000.0, 0.010, 0.015),
    (30000.0, 0.004, 0.006),
    (40000.0, 0.001, 0.001),
    (50000.0, 0.000, 0.000),
    (60000.0, 0.000, 0.000),
    (70000.0, 0.000, 0.000),
];


pub fn get_pressure(altitude: f32) -> f32 {
    if altitude < ATMOSPHERE[0].0 {
        return ATMOSPHERE[0].1
    }
    for ((h1, p1, _), (h2, p2, _)) in
        zip(&ATMOSPHERE[..ATMOSPHERE.len()-1], &ATMOSPHERE[1..]) {
        if altitude < *h2 {
            let f = (altitude - h1) / (h2 - h1);
            return f * (p2 - p1) + p1
        }
    }
    ATMOSPHERE[ATMOSPHERE.len()-1].1
}