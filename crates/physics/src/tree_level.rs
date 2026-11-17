//! Allocation-free arithmetic shared by the CPU and CUDA Oxide kernels.
pub fn matrix_element_squared(p: u32, s: f64, t: f64, u: f64, alpha: f64) -> f64 {
    let s2 = s * s;
    let t2 = t * t;
    let u2 = u * u;
    let g = 4. * core::f64::consts::PI * alpha;
    let g4 = g * g;
    g4 * match p {
        0 => 2. * (t2 + u2) / s2,
        1 => 4. / 9. * (s2 + u2) / t2,
        2 => 4. / 9. * ((s2 + u2) / t2 + (s2 + t2) / u2) - 8. / 27. * s2 / (u * t),
        3 => 4. / 9. * (t2 + u2) / s2,
        4 => 4. / 9. * ((s2 + u2) / t2 + (t2 + u2) / s2) - 8. / 27. * u2 / (s * t),
        5 => (s2 + u2) / t2 - 4. / 9. * (s2 + u2) / (s * u),
        6 => 32. / 27. * (u2 + t2) / (u * t) - 8. / 3. * (u2 + t2) / s2,
        7 => 1. / 6. * (u2 + t2) / (u * t) - 3. / 8. * (u2 + t2) / s2,
        _ => 4.5 * (3. - u * t / s2 - u * s / t2 - s * t / u2),
    }
}
