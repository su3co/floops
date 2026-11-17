//! Reproducible, memory-resident CUDA scattering campaign and provenance receipt.
use clap::Parser;
use floops::{
    cuda::{CudaResult, GpuScattering, BYTES_PER_EVENT},
    scattering::{self, Event, ScatterRequest},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 0)]
    gpu: usize,
    /// Accumulated CUDA-event time, excluding compilation, host copies and cooling.
    #[arg(long, default_value_t = 1200.)]
    seconds: f64,
    /// Omit to allocate all free VRAM except the reserve and reduction scratch.
    #[arg(long)]
    events: Option<usize>,
    #[arg(long, default_value_t = 2.)]
    reserve_gib: f64,
    #[arg(long, default_value_t = 2000.)]
    energy: f64,
    #[arg(long, default_value_t = 0.118)]
    alpha: f64,
    #[arg(long, default_value_t = 0.9)]
    cos_cut: f64,
    #[arg(long, default_value_t = 20261117)]
    seed: u64,
    #[arg(long, default_value = "output/cuda-run")]
    out: PathBuf,
    #[arg(long)]
    validate_only: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Telemetry {
    wall_seconds: f64,
    temperature_c: f64,
    memory_used_mib: f64,
    memory_total_mib: f64,
    utilization_percent: f64,
    power_w: f64,
}
fn telemetry(gpu: usize, wall_seconds: f64) -> CudaResult<Telemetry> {
    let out = Command::new("nvidia-smi")
        .args([
            "-i",
            &gpu.to_string(),
            "--query-gpu=temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()?;
    if !out.status.success() {
        return Err("nvidia-smi telemetry failed".into());
    }
    let values = String::from_utf8(out.stdout)?
        .trim()
        .split(',')
        .map(|s| s.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 5 {
        return Err("Unexpected GPU telemetry response".into());
    }
    Ok(Telemetry {
        wall_seconds,
        temperature_c: values[0],
        memory_used_mib: values[1],
        memory_total_mib: values[2],
        utilization_percent: values[3],
        power_w: values[4],
    })
}
#[derive(Serialize, Deserialize)]
struct Snapshot {
    events: u64,
    gpu_seconds: f64,
    wall_seconds: f64,
    cross_section_pb: f64,
    standard_error_pb: f64,
    histogram: Vec<f64>,
    histogram_error: Vec<f64>,
    sample: Vec<Event>,
    telemetry: Telemetry,
}
#[derive(Serialize, Deserialize)]
struct Receipt {
    schema_version: u32,
    state: String,
    backend: String,
    device: String,
    revision: String,
    started_unix_s: u64,
    requested_gpu_seconds: f64,
    gpu_seconds: f64,
    wall_seconds: f64,
    events: u64,
    batches: u64,
    resident_events: usize,
    event_bytes: usize,
    resident_event_bytes: usize,
    total_vram_bytes: usize,
    reserve_gib: f64,
    validation_max_relative_error: f64,
    request: ScatterRequest,
    reference_cross_section_pb: f64,
    reference_histogram: Vec<f64>,
    snapshots: Vec<Snapshot>,
    telemetry: Vec<Telemetry>,
    note: String,
}
fn write_json(path: &Path, data: &impl Serialize) -> CudaResult<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer(&mut file, data)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn reference(request: &ScatterRequest) -> Vec<f64> {
    let width = 2. * request.cos_cut / 60.;
    (0..60)
        .map(|b| {
            let low = -request.cos_cut + b as f64 * width;
            let h = width / 2000.;
            (0..=2000)
                .map(|i| {
                    let factor = if i == 0 || i == 2000 {
                        1.
                    } else if i % 2 == 0 {
                        2.
                    } else {
                        4.
                    };
                    factor
                        * scattering::differential_pb(
                            request.process,
                            request.energy,
                            low + i as f64 * h,
                            request.alpha,
                        )
                })
                .sum::<f64>()
                * h
                / 3.
        })
        .collect()
}
fn main() -> CudaResult<()> {
    let args = Args::parse();
    if !args.seconds.is_finite()
        || args.seconds <= 0.
        || !args.reserve_gib.is_finite()
        || args.reserve_gib < 1.
    {
        return Err("Use positive GPU seconds and reserve at least 1 GiB".into());
    }
    let request = ScatterRequest {
        energy: args.energy,
        alpha: args.alpha,
        cos_cut: args.cos_cut,
        seed: args.seed,
        display: 0,
        ..Default::default()
    };
    request.validate()?;
    let gpu = GpuScattering::new(args.gpu)?;
    let error = gpu.validate()?;
    println!("CUDA validation passed: all 9 channels, max relative difference {error:.3e}");
    if args.validate_only {
        return Ok(());
    }
    if args.out.join("result.json").exists() {
        return Err("Output receipt already exists; choose a new output directory".into());
    }
    fs::create_dir_all(&args.out)?;
    // Allow the host's normal idle-work supervisor to yield to this CUDA context.
    std::thread::sleep(Duration::from_secs(5));
    let (free, total) = gpu.context.mem_get_info()?;
    let reserve = (args.reserve_gib * 1073741824.) as usize + 16 * 1024 * 1024;
    let capacity = free
        .checked_sub(reserve)
        .ok_or("Insufficient free VRAM for reserve")?
        / BYTES_PER_EVENT;
    let count = args.events.unwrap_or(capacity / 256 * 256);
    if count > capacity {
        return Err("Requested batch would consume the VRAM reserve".into());
    }
    let mut buffer = gpu.allocate(count)?;
    gpu.context.synchronize()?;
    let theory = reference(&request);
    let mut receipt=Receipt {
        schema_version:1,state:"running".into(),backend:"Rust + CUDA FP64".into(),device:gpu.context.name()?,
        revision:std::env::var("FLOOPS_REVISION").unwrap_or_else(|_|"working-tree".into()),
        started_unix_s:SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),requested_gpu_seconds:args.seconds,
        gpu_seconds:0.,wall_seconds:0.,events:0,batches:0,resident_events:count,event_bytes:BYTES_PER_EVENT,
        resident_event_bytes:count*BYTES_PER_EVENT,total_vram_bytes:total,reserve_gib:args.reserve_gib,
        validation_max_relative_error:error,reference_cross_section_pb:theory.iter().sum(),reference_histogram:theory,
        request:request.clone(),snapshots:vec![],telemetry:vec![],
        note:"Independent weighted massless tree-level gg -> gg events; fixed alpha_s, no PDFs/showers/hadronization/detector. Reuses a fully populated resident event buffer with disjoint RNG indices. CUDA-event seconds measure generation and histogram reduction only. Animation is a sampled momentum visualization, not physical elapsed time.".into(),
    };
    println!(
        "{}: {} resident events, {:.3} GiB event data / {:.3} GiB card",
        receipt.device,
        count,
        receipt.resident_event_bytes as f64 / 1073741824.,
        total as f64 / 1073741824.
    );
    let start = Instant::now();
    let mut sums = [0.; 60];
    let mut squares = [0.; 60];
    let mut progress = File::create(args.out.join("progress.jsonl"))?;
    let mut next_snapshot = 0.;
    while receipt.gpu_seconds < args.seconds {
        let mut metrics = telemetry(args.gpu, start.elapsed().as_secs_f64())?;
        // Keep the allocation and event indices while cooling; cooling never counts
        // toward requested GPU execution time. Only this process is controlled.
        if metrics.temperature_c >= 84. {
            println!("Cooling at {} C", metrics.temperature_c);
            while metrics.temperature_c > 75. {
                receipt.telemetry.push(metrics);
                std::thread::sleep(Duration::from_secs(5));
                metrics = telemetry(args.gpu, start.elapsed().as_secs_f64())?;
            }
        }
        let offset = receipt.events;
        let stats = gpu.run(&mut buffer, &request, offset)?;
        receipt.events += stats.events;
        receipt.batches += 1;
        receipt.gpu_seconds += stats.gpu_seconds;
        receipt.wall_seconds = start.elapsed().as_secs_f64();
        for i in 0..60 {
            sums[i] += stats.histogram_sum[i];
            squares[i] += stats.histogram_square_sum[i];
        }
        metrics = telemetry(args.gpu, receipt.wall_seconds)?;
        receipt.telemetry.push(metrics.clone());
        let n = receipt.events as f64;
        let sigma = sums.iter().sum::<f64>() / n;
        let stderr = ((squares.iter().sum::<f64>() / n - sigma * sigma).max(0.) / (n - 1.)).sqrt();
        let line = serde_json::json!({"batch":receipt.batches,"events":receipt.events,"gpu_seconds":receipt.gpu_seconds,"wall_seconds":receipt.wall_seconds,"cross_section_pb":sigma,"standard_error_pb":stderr,"telemetry":metrics});
        serde_json::to_writer(&mut progress, &line)?;
        progress.write_all(b"\n")?;
        progress.flush()?;
        if receipt.gpu_seconds >= next_snapshot || receipt.gpu_seconds >= args.seconds {
            let sample = gpu.sample(&buffer, &request, offset, count.min(256))?;
            receipt.snapshots.push(Snapshot {
                events: receipt.events,
                gpu_seconds: receipt.gpu_seconds,
                wall_seconds: receipt.wall_seconds,
                cross_section_pb: sigma,
                standard_error_pb: stderr,
                histogram: sums.iter().map(|s| s / n).collect(),
                histogram_error: sums
                    .iter()
                    .zip(squares)
                    .map(|(s, q)| ((q / n - (s / n).powi(2)).max(0.) / (n - 1.)).sqrt())
                    .collect(),
                sample,
                telemetry: metrics,
            });
            write_json(&args.out.join("result.json"), &receipt)?;
            println!(
                "{:.1}/{:.1} GPU s | {} events | {:.6} +/- {:.6} pb",
                receipt.gpu_seconds, args.seconds, receipt.events, sigma, stderr
            );
            next_snapshot = receipt.gpu_seconds + 20.;
        }
    }
    receipt.state = "complete".into();
    write_json(&args.out.join("result.json"), &receipt)?;
    println!(
        "Complete: {} events, {:.3} GPU s, {:.3} wall s",
        receipt.events, receipt.gpu_seconds, receipt.wall_seconds
    );
    Ok(())
}
