use crate::overlap_save::OverlapSave;

pub use rustfft::num_complex::Complex32;

pub struct FftPipeline {
    os: OverlapSave,
}

impl FftPipeline {
    pub fn new(n: usize, v: usize) -> Self {
        Self {
            os: OverlapSave::new(n, v, true),
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8], out: &mut Vec<Vec<Complex32>>) {
        // Each sample is 2 bytes (little-endian int16)
        let mut samples = Vec::with_capacity(bytes.len() / 2);

        for chunk in bytes.chunks_exact(2) {
            let raw = i16::from_le_bytes([chunk[0], chunk[1]]);
            let v = raw as f32 / 32768.0;   // normalize to [-1, 1)
            samples.push(v);
        }
        // TEMP: print first 16 raw samples once
        static PRINTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !PRINTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            println!("First 16 raw samples: {:?}", &samples[..16]);
        }
        
        self.os.process_fft(&samples, out);
    }
}