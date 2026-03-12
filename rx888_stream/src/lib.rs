// rx888_stream/src/lib.rs

pub mod device;
pub mod init;
pub mod stream_manager;

pub use crate::device::{Rx888Device, Rx888Error, Rx888Result, DeviceMode};
pub use crate::init::SampleRate;
pub use crate::stream_manager::StreamManager;

use pyo3::prelude::*;

#[pyclass]
pub struct PyRx888Device {
    inner: Rx888Device,
}

#[pymethods]
impl PyRx888Device {
    #[new]
    pub fn new() -> PyResult<Self> {
        let dev = Rx888Device::open(0)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner: dev })
    }

    fn set_vga(&mut self, code: u16) -> PyResult<()> {
        self.inner
            .set_gain_from_python(11, code)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn set_attenuator(&mut self, att: u16) -> PyResult<()> {
        self.inner
            .set_gain_from_python(10, att)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymodule]
fn rx888_stream(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyRx888Device>()?;
    Ok(())
}