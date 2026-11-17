use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use floops::{amplitude, decay, diagram, lattice, scattering};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Parser)]
#[command(
    version,
    about = "Rust particle experiments: events, amplitudes, SU(3), and diagrams"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long, default_value = "127.0.0.1:8631")]
        bind: String,
        #[arg(long, default_value = "web/dist")]
        web: PathBuf,
    },
    Scatter {
        #[arg(long)]
        input: Option<PathBuf>,
    },
    Decay {
        #[arg(long)]
        input: Option<PathBuf>,
    },
    Lattice {
        #[arg(long)]
        input: Option<PathBuf>,
    },
    Amplitude {
        #[arg(long)]
        input: PathBuf,
    },
    Diagrams {
        #[arg(long, default_value_t = 4)]
        legs: usize,
    },
    Catalogue,
    Benchmark {
        #[arg(long, default_value_t = 1_000_000)]
        events: usize,
        #[arg(long, default_value_t = 3)]
        repeats: usize,
    },
}
fn input<T: DeserializeOwned + Default>(
    path: Option<PathBuf>,
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(match path {
        Some(p) => serde_json::from_slice(&std::fs::read(p)?)?,
        None => T::default(),
    })
}
fn print<T: serde::Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(std::io::stdout(), value)?;
    println!();
    Ok(())
}
type Gate = Arc<Semaphore>;
type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
async fn compute<T: serde::Serialize + Send + 'static>(
    gate: Gate,
    f: impl FnOnce() -> floops::Result<T> + Send + 'static,
) -> ApiResult {
    let permit = gate.try_acquire_owned().map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error":"Two calculations are already running. Retry after they finish."})),
        )
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        f().and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string()))
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error":"Calculation worker failed"})),
        )
    })?
    .map(Json)
    .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error":e}))))
}
async fn scatter(State(g): State<Gate>, Json(r): Json<scattering::ScatterRequest>) -> ApiResult {
    compute(g, move || scattering::simulate(&r)).await
}
async fn decay_run(State(g): State<Gate>, Json(r): Json<decay::DecayRequest>) -> ApiResult {
    compute(g, move || decay::simulate(&r)).await
}
async fn lattice_run(State(g): State<Gate>, Json(r): Json<lattice::LatticeRequest>) -> ApiResult {
    compute(g, move || lattice::simulate(&r)).await
}
async fn amp(State(g): State<Gate>, Json(r): Json<amplitude::AmplitudeRequest>) -> ApiResult {
    compute(g, move || amplitude::evaluate(&r)).await
}
#[derive(serde::Deserialize)]
struct DiagramRequest {
    process: Option<scattering::Process>,
    legs: Option<usize>,
}
async fn diagrams(Json(r): Json<DiagramRequest>) -> ApiResult {
    if let Some(p) = r.process {
        Ok(Json(json!({"diagrams":diagram::process_diagrams(p)})))
    } else {
        diagram::gluon_trees(r.legs.unwrap_or(4))
            .map(|trees| Json(json!({"count":trees.len(),"trees":trees})))
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error":e}))))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command.unwrap_or(Command::Serve {
        bind: "127.0.0.1:8631".into(),
        web: "web/dist".into(),
    }) {
        Command::Serve { bind, web } => {
            if !web.join("index.html").exists() {
                eprintln!("Web build missing. Run cd web && npm ci && npm run build, or use Vite in development.");
            }
            let app=Router::new().route("/api/health",get(||async {Json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION"),"engine":"Rust","threads":rayon::current_num_threads()}))}))
                .route("/api/catalogue",get(||async {Json(decay::catalogue())}))
                .route("/api/processes",get(||async {Json(scattering::Process::all().iter().map(|p|json!({"id":p,"label":p.label(),"species":p.species()})).collect::<Vec<_>>())}))
                .route("/api/scatter",post(scatter)).route("/api/decay",post(decay_run)).route("/api/lattice",post(lattice_run)).route("/api/amplitude",post(amp)).route("/api/diagrams",post(diagrams))
                .layer(DefaultBodyLimit::max(512_000)).with_state(Arc::new(Semaphore::new(2)))
                .fallback_service(ServeDir::new(&web).not_found_service(ServeFile::new(web.join("index.html"))));
            let listener = tokio::net::TcpListener::bind(&bind).await?;
            eprintln!("Particle laboratory: http://{bind}");
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
        Command::Scatter { input: p } => print(&scattering::simulate(&input(p)?)?)?,
        Command::Decay { input: p } => print(&decay::simulate(&input(p)?)?)?,
        Command::Lattice { input: p } => print(&lattice::simulate(&input(p)?)?)?,
        Command::Amplitude { input: p } => {
            let req = serde_json::from_slice(&std::fs::read(p)?)?;
            print(&amplitude::evaluate(&req)?)?;
        }
        Command::Diagrams { legs } => print(&diagram::gluon_trees(legs)?)?,
        Command::Catalogue => print(&decay::catalogue())?,
        Command::Benchmark { events, repeats } => {
            if repeats == 0 || repeats > 20 {
                return Err("repeats must be 1–20".into());
            }
            let req = scattering::ScatterRequest {
                events,
                display: 0,
                ..Default::default()
            };
            let _ = scattering::simulate(&scattering::ScatterRequest {
                events: 4096,
                display: 0,
                ..req.clone()
            })?;
            let mut serial = vec![];
            let mut parallel = vec![];
            let mut checksum = 0.;
            for _ in 0..repeats {
                let a = std::hint::black_box(scattering::simulate_with_parallel(&req, false)?);
                let b = std::hint::black_box(scattering::simulate_with_parallel(&req, true)?);
                if a.histogram != b.histogram {
                    return Err("Parallel reproducibility mismatch".into());
                }
                checksum = b.cross_section_pb;
                serial.push(a.elapsed_ms);
                parallel.push(b.elapsed_ms);
            }
            serial.sort_by(f64::total_cmp);
            parallel.sort_by(f64::total_cmp);
            print(
                &json!({"events":events,"repeats":repeats,"threads":rayon::current_num_threads(),"serial_ms":serial,"parallel_ms":parallel,"speedup":serial[repeats/2]/parallel[repeats/2],"parallel_events_per_second":events as f64/(parallel[repeats/2]/1000.),"cross_section_pb":checksum,"reproducible":true,"scope":"Massless weighted gg → gg event generation and histogram reduction, no shower or detector"}),
            )?;
        }
    }
    Ok(())
}
