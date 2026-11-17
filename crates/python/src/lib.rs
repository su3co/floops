use physics_core::{
    amplitude as amplitudes, decay as decays, diagram, lattice as lattices, scattering,
};
use pyo3::{exceptions::PyValueError, prelude::*};
fn json_result<T: serde::Serialize>(r: physics_core::Result<T>) -> PyResult<String> {
    serde_json::to_string(&r.map_err(PyValueError::new_err)?)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}
#[pyfunction]
fn scatter(py: Python<'_>, request_json: &str) -> PyResult<String> {
    let req: scattering::ScatterRequest =
        serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    py.detach(|| json_result(scattering::simulate(&req)))
}
#[pyfunction]
fn decay(py: Python<'_>, request_json: &str) -> PyResult<String> {
    let req: decays::DecayRequest =
        serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    py.detach(|| json_result(decays::simulate(&req)))
}
#[pyfunction]
fn lattice(py: Python<'_>, request_json: &str) -> PyResult<String> {
    let req: lattices::LatticeRequest =
        serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    py.detach(|| json_result(lattices::simulate(&req)))
}
#[pyfunction]
fn amplitude(request_json: &str) -> PyResult<String> {
    let req: amplitudes::AmplitudeRequest =
        serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_result(amplitudes::evaluate(&req))
}
#[pyfunction]
fn catalogue() -> PyResult<String> {
    json_result(Ok(decays::catalogue()))
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagramRequest {
    process: Option<scattering::Process>,
    legs: Option<usize>,
}
/// Return process diagrams as SVGs, or pure-gluon tree topologies as JSON.
#[pyfunction]
fn diagrams(py: Python<'_>, request_json: &str) -> PyResult<String> {
    let req: DiagramRequest =
        serde_json::from_str(request_json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    if req.process.is_some() && req.legs.is_some() {
        return Err(PyValueError::new_err("Supply process or legs, not both"));
    }
    py.detach(|| match req.process {
        Some(process) => json_result(Ok(diagram::process_diagrams(process))),
        None => json_result(diagram::gluon_trees(req.legs.unwrap_or(4))),
    })
}
#[pymodule]
fn floops(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(scatter, m)?)?;
    m.add_function(wrap_pyfunction!(decay, m)?)?;
    m.add_function(wrap_pyfunction!(lattice, m)?)?;
    m.add_function(wrap_pyfunction!(amplitude, m)?)?;
    m.add_function(wrap_pyfunction!(catalogue, m)?)?;
    m.add_function(wrap_pyfunction!(diagrams, m)?)?;
    Ok(())
}
