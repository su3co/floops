use crate::{scattering::Process, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "children")]
pub enum Tree {
    Leaf(usize),
    Vertex(Vec<Tree>),
}
/// Root on the final external gluon. Unordered partitions prevent duplicate diagrams.
fn currents(labels: &[usize]) -> Vec<Tree> {
    if labels.len() == 1 {
        return vec![Tree::Leaf(labels[0])];
    }
    let mut result = vec![];
    for count in [2, 3] {
        fn partitions(
            labels: &[usize],
            at: usize,
            count: usize,
            groups: &mut Vec<Vec<usize>>,
            out: &mut Vec<Vec<Vec<usize>>>,
        ) {
            if at == labels.len() {
                if groups.len() == count {
                    out.push(groups.clone());
                }
                return;
            }
            for k in 0..groups.len() {
                groups[k].push(labels[at]);
                partitions(labels, at + 1, count, groups, out);
                groups[k].pop();
            }
            if groups.len() < count {
                groups.push(vec![labels[at]]);
                partitions(labels, at + 1, count, groups, out);
                groups.pop();
            }
        }
        let mut groups = vec![];
        partitions(labels, 0, count, &mut vec![], &mut groups);
        for partition in groups {
            let mut combinations: Vec<Vec<Tree>> = vec![vec![]];
            for group in partition {
                let children = currents(&group);
                let mut next = vec![];
                for prefix in &combinations {
                    for child in &children {
                        let mut p = prefix.clone();
                        p.push(child.clone());
                        next.push(p);
                    }
                }
                combinations = next;
            }
            result.extend(combinations.into_iter().map(Tree::Vertex));
        }
    }
    result
}
pub fn gluon_trees(legs: usize) -> Result<Vec<Tree>> {
    if !(3..=6).contains(&legs) {
        return Err("Tree enumeration supports 3–6 external gluons".into());
    }
    Ok(currents(&(0..legs - 1).collect::<Vec<_>>())
        .into_iter()
        .map(|t| Tree::Vertex(vec![Tree::Leaf(legs - 1), t]))
        .collect())
}

#[derive(Serialize, Deserialize)]
pub struct Diagram {
    pub channel: String,
    pub mediator: String,
    pub external: [i32; 4],
    pub svg: String,
}
pub fn process_diagrams(p: Process) -> Vec<Diagram> {
    let channels: &[(&str, &str)] = match p {
        Process::ElectronMuon => &[("s", "photon")],
        Process::QuarkDifferent => &[("t", "gluon")],
        Process::QuarkSame => &[("t", "gluon"), ("u", "gluon")],
        Process::Annihilation => &[("s", "gluon")],
        Process::QuarkAntiquark => &[("s", "gluon"), ("t", "gluon")],
        Process::QuarkGluon => &[("s", "quark"), ("u", "quark"), ("t", "gluon")],
        Process::QuarkToGluon | Process::GluonToQuark => {
            &[("t", "quark"), ("u", "quark"), ("s", "gluon")]
        }
        Process::GluonGluon => &[
            ("s", "gluon"),
            ("t", "gluon"),
            ("u", "gluon"),
            ("contact", "gluon"),
        ],
    };
    channels
        .iter()
        .map(|(c, m)| Diagram {
            channel: (*c).into(),
            mediator: (*m).into(),
            external: p.species(),
            svg: svg(p.species(), c, m),
        })
        .collect()
}
fn species_label(id: i32) -> &'static str {
    match id {
        11 => "e⁻",
        -11 => "e⁺",
        13 => "μ⁻",
        -13 => "μ⁺",
        21 => "g",
        x if x < 0 => "q̄",
        _ => "q",
    }
}
fn line(a: (f64, f64), b: (f64, f64), kind: &str, arrow: bool) -> String {
    if kind == "quark" || kind == "fermion" {
        return format!(
            "<path d=\"M {} {} L {} {}\" {}/>",
            a.0,
            a.1,
            b.0,
            b.1,
            if arrow {
                "marker-end=\"url(#arrow)\""
            } else {
                ""
            }
        );
    }
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let norm = dx.hypot(dy);
    let mut points = String::new();
    for k in 0..=100 {
        let t = k as f64 / 100.;
        let phase = t * std::f64::consts::TAU * if kind == "gluon" { 9. } else { 6. };
        let offset = phase.sin() * 3.;
        let along = if kind == "gluon" {
            phase.cos() * 2. * (std::f64::consts::PI * t).sin()
        } else {
            0.
        };
        points.push_str(&format!(
            "{:.2},{:.2} ",
            a.0 + dx * t - dy / norm * offset + dx / norm * along,
            a.1 + dy * t + dx / norm * offset + dy / norm * along
        ));
    }
    format!("<polyline points=\"{points}\"/>")
}
pub fn svg(ext: [i32; 4], channel: &str, mediator: &str) -> String {
    let tips = [(24., 28.), (24., 132.), (216., 28.), (216., 132.)];
    let vertices = match channel {
        "t" => [(120., 48.), (120., 112.), (120., 48.), (120., 112.)],
        "u" => [(94., 53.), (146., 107.), (146., 107.), (94., 53.)],
        "contact" => [(120., 80.); 4],
        _ => [(88., 80.), (88., 80.), (152., 80.), (152., 80.)],
    };
    let mut s=String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 240 172\" role=\"img\"><title>Tree-level Feynman diagram</title><defs><marker id=\"arrow\" markerWidth=\"5\" markerHeight=\"5\" refX=\"3\" refY=\"2.5\" orient=\"auto\"><path d=\"M0,0 L5,2.5 L0,5\" fill=\"#254d75\"/></marker></defs><g fill=\"none\" stroke=\"#254d75\" stroke-width=\"1.6\">");
    for i in 0..4 {
        let (a, b) = if (i < 2) == (ext[i] > 0) {
            (tips[i], vertices[i])
        } else {
            (vertices[i], tips[i])
        };
        s.push_str(&line(
            a,
            b,
            if ext[i] == 21 { "gluon" } else { "fermion" },
            ext[i] != 21,
        ));
    }
    if channel != "contact" {
        let (a, b) = if channel == "s" {
            (vertices[0], vertices[2])
        } else {
            (vertices[0], vertices[1])
        };
        s.push_str(&line(a, b, mediator, mediator == "quark"));
    }
    s.push_str(
        "</g><g fill=\"#17293d\" font-size=\"14\" font-family=\"serif\" text-anchor=\"middle\">",
    );
    for i in 0..4 {
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\">{}</text>",
            tips[i].0,
            tips[i].1 + if i % 2 == 0 { -9. } else { 20. },
            species_label(ext[i])
        ));
    }
    s.push_str("</g></svg>");
    s
}
