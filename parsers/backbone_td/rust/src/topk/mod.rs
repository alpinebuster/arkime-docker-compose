use pyo3::{
    types::{PyAnyMethods, PyModule, PyModuleMethods},
    Bound, PyResult, Python,
};

pub mod small_flow_sampling;
pub mod holt;

pub fn register_module(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let child_module = PyModule::new_bound(py, "topk")?;

    child_module.add_class::<small_flow_sampling::Sampler>()?;

    m.add_submodule(&child_module)?;

    // We need to manually add the module to sys.modules to make 
    // `from src.src_rust import topk` work.
    py.import_bound("sys")?
        .getattr("modules")?
        .set_item("src.src_rust.topk", child_module)?;

    Ok(())
}
