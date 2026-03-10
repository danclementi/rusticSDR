use rustfft::{FftPlanner, num_complex::Complex32};
use rustfft::num_traits::Zero;

pub struct OverlapSave {
    n: usize,
    v: usize,
    m: usize,
    l: usize,
    prev: Vec<f32>,
    pending: Vec<f32>,
    window: Option<Vec<f32>>,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
}

impl OverlapSave {
    pub fn new(n: usize, v: usize, use_window: bool) -> Self {
        assert!(n % v == 0);
        let m = n / v;
        let l = n - m;

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n);

        let window = if use_window {
            Some((0..n).map(|i| {
                let x = (i as f32) / (n as f32 - 1.0);
                (std::f32::consts::PI * x).sin().powi(2) // Hann
            }).collect())
        } else {
            None
        };

        Self {
            n,
            v,
            m,
            l,
            prev: vec![0.0; m],
            pending: Vec::new(),
            window,
            fft,
        }
    }

    pub fn process_fft(&mut self, input: &[f32], out: &mut Vec<Vec<Complex32>>) {
        // append new samples
        self.pending.extend_from_slice(input);

        let mut idx = 0;
        while idx + self.l <= self.pending.len() {
            let new = &self.pending[idx..idx + self.l];

            let mut block = Vec::with_capacity(self.n);
            block.extend_from_slice(&self.prev);
            block.extend_from_slice(new);

            self.prev.copy_from_slice(&block[self.n - self.m..]);

            if let Some(w) = &self.window {
                for (b, wv) in block.iter_mut().zip(w.iter()) {
                    *b *= *wv;
                }
            }

            let mut spectrum: Vec<Complex32> =
                block.into_iter().map(|x| Complex32::new(x, 0.0)).collect();

            self.fft.process(&mut spectrum);

            out.push(spectrum);

            idx += self.l;
        }

        // keep leftover
        self.pending.drain(0..idx);
    }
}