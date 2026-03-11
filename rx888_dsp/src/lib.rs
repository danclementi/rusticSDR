mod fft_pipeline;

pub use fft_pipeline::FftPipeline;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use numpy::PyArray1;
use numpy::PyArray1 as NpyArray1; // make sure `numpy = "0.21"` (or similar) is in Cargo.toml

use rx888_stream::{SampleRate, StreamManager, Rx888Error};

fn to_pyerr(e: Rx888Error) -> PyErr {
    PyRuntimeError::new_err(format!("RX-888 error: {e:?}"))
}

fn map_sample_rate(s: Option<&str>) -> Result<SampleRate, Rx888Error> {
    match s {
        None => Ok(SampleRate::Sps64_8M), // default
        Some("32.4M") | Some("32.4m") => Ok(SampleRate::Sps32_4M),
        Some("64.8M") | Some("64.8m") => Ok(SampleRate::Sps64_8M),
        Some("129.6M") | Some("129.6m") => Ok(SampleRate::Sps129_6M),
        Some(other) => Err(Rx888Error::Usb(format!("Unsupported sample rate: {other}"))),
    }
}

#[pyclass]
pub struct PyStreamManager {
    inner: StreamManager,
}

#[pymethods]
impl PyStreamManager {
    /// Create a new RX-888 stream.
    ///
    /// sample_rate: "32.4M", "64.8M", or "129.6M" (default: "64.8M")
    #[new]
    pub fn new(sample_rate: Option<&str>) -> PyResult<Self> {
        let rate = map_sample_rate(sample_rate).map_err(to_pyerr)?;
        let inner = StreamManager::new(rate).map_err(to_pyerr)?;
        Ok(Self { inner })
    }

    pub fn start(&mut self) -> PyResult<()> {
        self.inner.start().map_err(to_pyerr)?;
        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        self.inner.stop().map_err(to_pyerr)?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    /// Read `n_bytes` of raw IQ data (uint8) from the stream.
    pub fn read_samples<'py>(
        &mut self,
        py: Python<'py>,
        n_bytes: usize,
    ) -> PyResult<&'py PyBytes> {
        let mut buf = vec![0u8; n_bytes];
        self.inner.read_samples(&mut buf).map_err(to_pyerr)?;
        Ok(PyBytes::new(py, &buf))
    }
}



#[pyclass]
pub struct PyFftPipeline {
    inner: FftPipeline,
}

#[pymethods]
impl PyFftPipeline {
    /// Create a new FFT pipeline with the given fft_size.
    #[new]
    pub fn new(fft_size: usize) -> Self {
        Self {
            inner: FftPipeline::new(fft_size),
        }
    }

    /// Process one frame of i16 samples (as Python bytes) and return magnitude spectrum as numpy.float32 array.
    pub fn process<'py>(
        &mut self,
        py: Python<'py>,
        samples: &PyBytes,
    ) -> PyResult<&'py NpyArray1<f32>> {
        let buf = samples.as_bytes();

        if buf.len() % 2 != 0 {
            return Err(PyRuntimeError::new_err("Input buffer length must be even (i16 samples)"));
        }

        let n_samples = buf.len() / 2;
        if n_samples != self.inner.fft_size() {
            return Err(PyRuntimeError::new_err(format!(
                "Expected {} samples, got {}",
                self.inner.fft_size(),
                n_samples
            )));
        }

        // Interpret bytes as i16
        let mut frame = Vec::with_capacity(n_samples);
        for chunk in buf.chunks_exact(2) {
            let v = i16::from_le_bytes([chunk[0], chunk[1]]);
            frame.push(v);
        }

        let mag = self.inner.process(&frame);

        // Return as NumPy array (zero-copy from Rust slice)
        let out = PyArray1::from_slice(py, mag);
        Ok(out)
    }

    pub fn fft_size(&self) -> usize {
        self.inner.fft_size()
    }

    pub fn spectrum_len(&self) -> usize {
        self.inner.spectrum_len()
    }
}

#[pymodule]
fn rx888_dsp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStreamManager>()?;
    m.add_class::<PyFftPipeline>()?;   // <-- REQUIRED
    Ok(())
}