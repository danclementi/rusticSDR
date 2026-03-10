pub mod device;
pub mod rx888;
pub mod fx3;

// use pyo3::prelude::*;
// use pyo3::exceptions::{PyRuntimeError, PyValueError};

// mod device;
// pub use device::Rx888Device;
// use crate::device::DeviceMode;


// use crate::stream_manager::StreamManager;
// use crate::pipeline::Complex32;   // adjust path if needed

// #[pyclass]
// pub struct PyStreamManager {
//     inner: StreamManager,
// }


// #[pymodule]
// fn rx888_stream(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_class::<PyRx888Device>()?;
//     Ok(())
// }


// #[pyclass]
// pub struct PyRx888Device {
//     inner: Rx888Device,
// }


// #[pymethods]
// impl PyStreamManager {
//     #[new]
//     fn new(n: usize, v: usize, chunk_samples: usize) -> PyResult<Self> {
//         Ok(PyStreamManager {
//             inner: StreamManager::start(n, v, chunk_samples),
//         })
//     }

//     fn stop(&self) {
//         self.inner.stop();
//     }

//     fn try_recv_frame(&self) -> Option<Vec<Complex32>> {
//         self.inner.try_recv_frame()
//     }

//     fn stats(&self) -> (u64, u64) {
//         self.inner.stats()
//     }
// }

// impl PyRx888Device {
//     #[new]
//     fn new(index: usize) -> PyResult<Self> {
//         Ok(PyRx888Device {
//             inner: Rx888Device::open(index)
//                 .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?
//         })
//     }

//     fn get_mode(&self) -> PyResult<String> {
//         Ok(match self.inner.mode() {
//             DeviceMode::Bootloader => "bootloader".to_string(),
//             DeviceMode::Runtime => "runtime".to_string(),
//         })
//     }

//     pub fn reset_fx3(&mut self) -> PyResult<()> {
//         self.inner
//             .reset_fx3()
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     pub fn read_debug(&mut self) -> PyResult<Vec<u8>> {
//         self.inner
//             .read_debug()
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     pub fn start_adc(&mut self, freq: u32) -> PyResult<()> {
//         self.inner
//             .start_adc(freq)
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     pub fn start_fx3(&mut self) -> PyResult<()> {
//         self.inner
//             .start_fx3()
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     pub fn tuner_init(&mut self) -> PyResult<()> {
//         self.inner
//             .tuner_init()
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     pub fn tuner_tune(&mut self, freq: u64) -> PyResult<()> {
//         self.inner
//             .tuner_tune(freq)
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     /// Version that takes a raw argument ID (1,2,3,...) and maps it to ArgumentList.
//     pub fn set_gain(&mut self, arg: u16, value: u16) -> PyResult<()> {
//         use device::ArgumentList;

//         let arg_enum = match arg {
//             1 => ArgumentList::R82XX_ATTENUATOR,
//             2 => ArgumentList::R82XX_VGA,
//             3 => ArgumentList::R82XX_SIDEBAND,
//             4 => ArgumentList::R82XX_HARMONIC,
//             10 => ArgumentList::DAT31_ATT,
//             11 => ArgumentList::AD8340_VGA,
//             12 => ArgumentList::PRESELECTOR,
//             13 => ArgumentList::VHF_ATTENUATOR,
//             _ => return Err(PyValueError::new_err("Invalid argument ID")),
//         };

//         self.inner
//             .set_gain(arg_enum, value)
//             .map_err(|e| PyRuntimeError::new_err(format!("{:?}", e)))
//     }

//     fn upload_firmware(&mut self) -> PyResult<()> {
//         self.inner.download_fx3_firmware()
//             .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))
//     }
// }