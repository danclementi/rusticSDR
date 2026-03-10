use pyo3::prelude::*;
use pyo3::types::PyComplex;
use numpy::PyArray1;
use rustfft::num_complex::{Complex32, Complex64};


mod stream_manager;
mod pipeline;
mod fft_engine;
mod overlap_save;

use stream_manager::StreamManager;
// use pipeline::Complex32;

#[pyclass]
pub struct PyStreamManager {
    inner: StreamManager,
}

#[pymethods]
impl PyStreamManager {
    #[new]
    fn new(n: usize, v: usize, chunk_samples: usize) -> PyResult<Self> {
        Ok(PyStreamManager {
            inner: StreamManager::start(n, v, chunk_samples),
        })
    }

    fn stop(&self) {
        self.inner.stop();
    }

    fn try_recv_frame<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyArray1<Complex64>>> {
        if let Some(frame) = self.inner.try_recv_frame() {
            // Convert Complex32 → Complex64 (NumPy-compatible)
            let converted: Vec<Complex64> = frame
                .iter()
                .map(|c: &Complex32| Complex64::new(c.re as f64, c.im as f64))
                .collect();

            Ok(Some(PyArray1::from_vec(py, converted)))
        } else {
            Ok(None)
        }
    }


    fn stats(&self) -> (u64, u64) {
        self.inner.stats()
    }
}


#[pymodule]
fn rx888_dsp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStreamManager>()?;
    Ok(())
}

// use pyo3::prelude::*;
// use pyo3::types::PyComplex;

// mod pipeline;
// mod overlap_save;
// mod stream_manager;



// use crate::stream_manager::StreamManager;
// use num_complex::Complex32;

// #[pyclass]
// pub struct PyStreamManager {
//     inner: StreamManager,
// }

// #[pymethods]
// impl PyStreamManager {
//     /// Create and start the stream manager.
//     ///
//     /// n = FFT size
//     /// v = overlap factor
//     /// chunk_samples = number of int16 samples per read
//     #[new]
//     fn new(n: usize, v: usize, chunk_samples: usize) -> Self {
//         let inner = StreamManager::start(n, v, chunk_samples);
//         Self { inner }
//     }

//     /// Non-blocking poll for one FFT frame.
//     /// Returns a Python list of complex numbers or None.
//     fn poll_frame(&self, py: Python<'_>) -> PyResult<Option<Vec<PyObject>>> {
//         if let Some(frame) = self.inner.try_recv_frame() {
//             let pylist: Vec<PyObject> = frame
//                 .into_iter()
//                 .map(|c: Complex32| {
//                     PyComplex::from_doubles(py, c.re as f64, c.im as f64)
//                         .into_py(py)
//                 })
//                 // .map(|c: Complex32| {
//                 //     PyComplex::from_doubles(c.re as f64, c.im as f64)
//                 //         .into_py(py)
//                 // })
//                 .collect();
//             Ok(Some(pylist))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Return (bytes_in, frames_out)
//     fn stats(&self) -> PyResult<(u64, u64)> {
//         Ok(self.inner.stats())
//     }

//     /// Stop the producer + consumer threads
//     fn stop(&self) {
//         self.inner.stop();
//     }
// }

// #[pymodule]
// fn rx888_dsp(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_class::<PyStreamManager>()?;
//     Ok(())
// }