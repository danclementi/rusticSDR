use std::sync::Arc;
use realfft::{RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex32;

pub struct FftPipeline {
    fft_size: usize,
    r2c: Arc<dyn RealToComplex<f32>>,
    input: Vec<f32>,
    spectrum_c: Vec<Complex32>,
    spectrum_mag: Vec<f32>,
    scratch: Vec<Complex32>,
}

impl FftPipeline {
    pub fn new(fft_size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(fft_size);

        let input = vec![0.0f32; fft_size];
        let spectrum_c = r2c.make_output_vec();
        let spectrum_mag = vec![0.0f32; spectrum_c.len()];
        let scratch = r2c.make_scratch_vec();

        Self {
            fft_size,
            r2c,
            input,
            spectrum_c,
            spectrum_mag,
            scratch,
        }
    }

    /// Process one frame of real i16 samples.
    /// Returns a slice of magnitude spectrum (len = fft_size/2 + 1).
    pub fn process(&mut self, samples: &[i16]) -> &[f32] {
        assert_eq!(
            samples.len(),
            self.fft_size,
            "FftPipeline: input len must equal fft_size"
        );

        // Convert i16 → f32
        for (dst, src) in self.input.iter_mut().zip(samples.iter()) {
            *dst = *src as f32;
        }

        // Run real FFT
        self.r2c
            .process_with_scratch(&mut self.input, &mut self.spectrum_c, &mut self.scratch)
            .expect("FFT process failed");

        // Compute magnitude
        for (i, c) in self.spectrum_c.iter().enumerate() {
            self.spectrum_mag[i] = (c.re * c.re + c.im * c.im).sqrt();
        }

        &self.spectrum_mag
    }

    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    pub fn spectrum_len(&self) -> usize {
        self.spectrum_mag.len()
    }
}