use floops::{
    amplitude::*,
    decay, diagram,
    lattice::{Lattice, LatticeRequest},
    math::*,
    rng::Rng,
    scattering::*,
    su3::*,
};
use std::f64::consts::PI;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() < tol, "{a} != {b}, tolerance {tol}");
}

#[test]
fn lorentz_invariance_and_round_trip() {
    let p = FourVector::new(4., 1., 2., 1.);
    for b in [[0., 0., 0.], [0.2, -0.4, 0.7], [0., 0., 0.9999]] {
        let q = p.boost(b).unwrap();
        near(p.mass2(), q.mass2(), 1e-8);
        assert!((q.boost(b.map(|x| -x)).unwrap() - p).max_abs() < 1e-8);
    }
    assert!(p.boost([1., 0., 0.]).is_err());
}
#[test]
fn phase_space_preserves_four_momentum_and_mass() {
    let mut r = Rng::new(17);
    let parent = FourVector::rest(5.).boost([0.2, 0.4, 0.7]).unwrap();
    for _ in 0..2000 {
        let q = three_body(parent, [0.1, 0.5, 1.], &mut r).unwrap();
        assert!(
            (q.iter().copied().fold(FourVector::default(), |a, b| a + b) - parent).max_abs()
                < 1e-11
        );
        for (p, m) in q.iter().zip([0.1_f64, 0.5, 1.]) {
            near(p.mass2(), m * m, 1e-10);
        }
    }
    assert!(two_body(FourVector::rest(1.), [1., 1.], &mut r).is_err());
}
#[test]
fn massless_phase_space_has_known_energy_distribution() {
    let mut r = Rng::new(9);
    let mut x = 0.;
    let mut x2 = 0.;
    for _ in 0..30000 {
        let q = three_body(FourVector::rest(1.), [0.; 3], &mut r).unwrap();
        let e = 2. * q[0].e;
        x += e;
        x2 += e * e;
    }
    near(x / 30000., 2. / 3., 0.006);
    near(x2 / 30000., 0.5, 0.006);
}
#[test]
fn gell_mann_normalization_casimirs_and_jacobi() {
    let t = generators();
    let mut casimir = Matrix::zero();
    for a in 0..8 {
        near(t[a].trace().norm(), 0., 1e-14);
        casimir = casimir + t[a] * t[a];
        for b in 0..8 {
            near(
                (t[a] * t[b]).trace().re,
                if a == b { 0.5 } else { 0. },
                1e-14,
            );
        }
    }
    near((casimir - Matrix::identity() * (4. / 3.)).norm(), 0., 1e-14);
    near(structure_f(0, 1, 2), 1., 1e-14);
    for a in 0..8 {
        for b in 0..8 {
            let ca: f64 = (0..8)
                .flat_map(|c| (0..8).map(move |d| structure_f(a, c, d) * structure_f(b, c, d)))
                .sum();
            near(ca, if a == b { 3. } else { 0. }, 1e-12);
        }
    }
    for (a, b, c) in [(0, 3, 6), (1, 2, 7), (4, 5, 6)] {
        let comm = |a: Matrix, b: Matrix| a * b - b * a;
        near(
            (comm(t[a], comm(t[b], t[c]))
                + comm(t[b], comm(t[c], t[a]))
                + comm(t[c], comm(t[a], t[b])))
            .norm(),
            0.,
            1e-12,
        );
    }
}
#[test]
fn haar_and_updates_stay_su3() {
    let mut r = Rng::new(11);
    let mut mean = num_complex::Complex64::new(0., 0.);
    for _ in 0..1000 {
        let u = Matrix::haar(&mut r);
        near(u.unitary_error(), 0., 1e-12);
        near((u.det() - 1.).norm(), 0., 1e-12);
        mean += u.trace();
    }
    assert!((mean / 1000.).norm() < 0.1);
    let mut l = Lattice::new(3, true, &mut r).unwrap();
    for _ in 0..5 {
        l.sweep(5.7, 0.3, &mut r);
    }
    assert!(l
        .links
        .iter()
        .all(|u| u.unitary_error() < 1e-12 && (u.det() - 1.).norm() < 1e-12));
}
#[test]
fn lattice_local_action_matches_global_change() {
    let mut r = Rng::new(32);
    for n in [2, 3, 4] {
        let mut l = Lattice::new(n, true, &mut r).unwrap();
        for x in [0, 1, n * n] {
            for mu in 0..4 {
                let old = l.link(x, mu);
                let new = Matrix::haar(&mut r);
                let delta = -5.7 / 3. * ((new - old) * l.staple(x, mu)).trace().re;
                let before = l.action(5.7);
                l.links[4 * x + mu] = new;
                near(l.action(5.7) - before, delta, 1e-9);
            }
        }
    }
}
#[test]
fn lattice_observables_are_gauge_invariant() {
    let mut r = Rng::new(11);
    let mut cold = Lattice::new(3, false, &mut r).unwrap();
    near(cold.mean_plaquette(), 1., 1e-14);
    near(cold.action(5.7), 0., 1e-14);
    near(cold.wilson_loop(2, 2), 1., 1e-14);
    cold.sweep(5.7, 0.5, &mut r);
    let before = (
        cold.mean_plaquette(),
        cold.wilson_loop(2, 2),
        cold.polyakov(),
    );
    let g = (0..cold.volume())
        .map(|_| Matrix::haar(&mut r))
        .collect::<Vec<_>>();
    cold.gauge_transform(&g).unwrap();
    near(before.0, cold.mean_plaquette(), 1e-12);
    near(before.1, cold.wilson_loop(2, 2), 1e-12);
    near(before.2[0], cold.polyakov()[0], 1e-12);
    near(before.2[1], cold.polyakov()[1], 1e-12);
}
#[test]
fn qed_cross_section_matches_analytic_integral() {
    let a = 1. / 137.035999084;
    let e = 100.;
    let c = 0.9;
    let steps = 10000;
    let step = 2. * c / steps as f64;
    let sum: f64 = (0..=steps)
        .map(|i| {
            let factor = if i == 0 || i == steps {
                1.
            } else if i % 2 == 0 {
                2.
            } else {
                4.
            };
            factor * differential_pb(Process::ElectronMuon, e, -c + i as f64 * step, a)
        })
        .sum();
    let expected = PI * a * a / (e * e) * (c + c.powi(3) / 3.) * GEV2_TO_PB;
    near(sum * step / 3., expected, 1e-8);
}
#[test]
fn qcd_pdg_reference_values_and_detailed_balance() {
    let g4 = (4. * PI * 0.118).powi(2);
    let expected = [
        1.,
        20. / 9.,
        88. / 27.,
        2. / 9.,
        70. / 27.,
        55. / 9.,
        28. / 27.,
        7. / 48.,
        243. / 8.,
    ];
    for (p, e) in Process::all().into_iter().zip(expected) {
        near(
            matrix_element_squared(p, 1., -0.5, -0.5, 0.118) / g4,
            e,
            1e-12,
        );
    }
    for c in [-0.8, -0.2, 0.3, 0.9] {
        let s = 1.;
        let t = -(1. - c) / 2.;
        let u = -(1. + c) / 2.;
        near(
            matrix_element_squared(Process::QuarkToGluon, s, t, u, 0.1) * 9.,
            matrix_element_squared(Process::GluonToQuark, s, t, u, 0.1) * 64.,
            1e-12,
        );
    }
}
#[test]
fn batch_is_reproducible_and_mc_covers_analytic_result() {
    let r = ScatterRequest {
        process: Process::ElectronMuon,
        alpha: 1. / 137.,
        events: 100000,
        display: 16,
        ..Default::default()
    };
    let a = simulate_with_parallel(&r, false).unwrap();
    let b = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap()
        .install(|| simulate(&r))
        .unwrap();
    assert_eq!(a.histogram, b.histogram);
    assert_eq!(a.transverse_momentum_sum, b.transverse_momentum_sum);
    let c = r.cos_cut;
    let exact = PI * r.alpha.powi(2) / r.energy.powi(2) * (c + c.powi(3) / 3.) * GEV2_TO_PB;
    assert!((a.cross_section_pb - exact).abs() < 5. * a.standard_error_pb);
    for e in a.events {
        assert!((e.momenta[0] + e.momenta[1] - e.momenta[2] - e.momenta[3]).max_abs() < 1e-12);
    }
}
#[test]
fn parke_taylor_four_point_magnitude_and_cyclic_symmetry() {
    let q = com_momenta(10., 0.3, 0.8);
    let p = [-q[0], -q[1], q[2], q[3]];
    let a = mhv(&p, [0, 1], 1.).unwrap();
    let s = (p[0] + p[1]).mass2();
    let adjacent = (p[1] + p[2]).mass2();
    near(a.norm_sqr(), s * s / (adjacent * adjacent), 1e-12);
    let rotated = [p[1], p[2], p[3], p[0]];
    near((a - mhv(&rotated, [3, 0], 1.).unwrap()).norm(), 0., 1e-12);
    for i in 0..4 {
        for j in i + 1..4 {
            let bracket2 = bracket(spinor(p[i]).unwrap(), spinor(p[j]).unwrap()).norm_sqr();
            near(bracket2, (2. * p[i].dot(p[j])).abs(), 1e-10);
        }
    }
    assert!(mhv(&p, [0, 0], 1.).is_err());
}
#[test]
fn gluon_tree_counts_include_quartic_vertices() {
    for (n, count) in [(3, 1), (4, 4), (5, 25), (6, 220)] {
        assert_eq!(diagram::gluon_trees(n).unwrap().len(), count);
    }
    assert!(diagram::gluon_trees(7).is_err());
}

#[test]
fn multi_leg_amplitudes_have_correct_scaling_and_cyclic_symmetry() {
    for n in [5, 6, 12] {
        let mut p = vec![
            FourVector::new(-50., 0., 0., -50.),
            FourVector::new(-50., 0., 0., 50.),
        ];
        for k in 0..n - 2 {
            let phi = 0.4 + 2. * PI * k as f64 / (n - 2) as f64;
            let e = 100. / (n - 2) as f64;
            p.push(FourVector::new(e, e * phi.cos(), e * phi.sin(), 0.));
        }
        let a = mhv(&p, [0, 1], 1.).unwrap();
        let scaled: Vec<_> = p.iter().map(|v| *v * 3.).collect();
        let b = mhv(&scaled, [0, 1], 1.).unwrap();
        near((b / a).re, 3_f64.powi(4 - n as i32), 1e-10);
        p.rotate_left(1);
        near(
            (mhv(&p, [n - 1, 0], 1.).unwrap() / a - 1.).norm(),
            0.,
            1e-10,
        );
    }
}

#[test]
fn complex_amplitudes_color_sum_reproduces_pdg_gg_scattering() {
    for cosine in [-0.7, 0., 0.3, 0.85] {
        let p = com_momenta(100., cosine, 0.4);
        let all = [-p[0], -p[1], p[2], p[3]];
        let alpha = 0.118;
        let computed = four_gluon_averaged(&all, (4. * PI * alpha).sqrt()).unwrap();
        let reference = matrix_element_squared(
            Process::GluonGluon,
            10000.,
            -5000. * (1. - cosine),
            -5000. * (1. + cosine),
            alpha,
        );
        near(computed / reference, 1., 1e-10);
    }
}
#[test]
fn catalogue_and_decay_chains_conserve_charge_and_energy() {
    let model = decay::catalogue();
    model.validate().unwrap();
    let map: std::collections::HashMap<_, _> = model.particles.iter().map(|p| (p.pdg, p)).collect();
    for id in model
        .particles
        .iter()
        .filter(|p| !p.channels.is_empty())
        .map(|p| p.pdg)
    {
        let r = decay::DecayRequest {
            pdg: id,
            events: 100,
            display: 100,
            ..Default::default()
        };
        let result = decay::simulate(&r).unwrap();
        assert!(
            result.max_conservation_error < 1e-7,
            "{id}: {}",
            result.max_conservation_error
        );
        for e in result.events {
            let leaves: Vec<_> = e.nodes.iter().filter(|n| n.status != "decayed").collect();
            let sum = leaves
                .iter()
                .fold(FourVector::default(), |a, n| a + n.momentum);
            assert!((sum - e.nodes[0].momentum).max_abs() < 1e-7);
            assert_eq!(
                leaves
                    .iter()
                    .map(|n| map[&n.pdg].charge_thirds)
                    .sum::<i32>(),
                map[&id].charge_thirds
            );
            for n in &e.nodes {
                near(n.momentum.mass2(), map[&n.pdg].mass.powi(2), 1e-7);
            }
        }
    }
}
#[test]
fn muon_michel_spectrum_and_lifetime() {
    let r = decay::DecayRequest {
        pdg: 13,
        momentum: 0.,
        events: 30000,
        display: 0,
        ..Default::default()
    };
    let x = decay::simulate(&r).unwrap();
    let mean = x
        .energy_histogram
        .iter()
        .enumerate()
        .map(|(i, n)| *n as f64 * (i as f64 + 0.5) / 60.)
        .sum::<f64>()
        / r.events as f64;
    near(mean, 0.7, 0.006);
    near(x.mean_root_proper_time_s / 2.1969811e-6, 1., 0.025);
}
#[test]
fn missing_decay_modes_are_not_silently_renormalized() {
    let x = decay::simulate(&decay::DecayRequest {
        pdg: 443,
        momentum: 0.,
        events: 10000,
        display: 0,
        ..Default::default()
    })
    .unwrap();
    let missing = *x.channels.get("unmodeled").unwrap() as f64 / 10000.;
    near(missing, 1. - 0.05971 - 0.05961, 0.02);
}
#[test]
fn invalid_requests_are_rejected() {
    assert!(simulate(&ScatterRequest {
        energy: f64::NAN,
        ..Default::default()
    })
    .is_err());
    assert!(simulate(&ScatterRequest {
        cos_cut: 1.,
        ..Default::default()
    })
    .is_err());
    assert!(floops::lattice::simulate(&LatticeRequest {
        chains: 0,
        ..Default::default()
    })
    .is_err());
    assert!(decay::simulate(&decay::DecayRequest {
        pdg: 22,
        ..Default::default()
    })
    .is_err());
    let mut model = decay::catalogue();
    model.particles[0].mass = f64::NAN;
    assert!(model.validate().is_err());
}
