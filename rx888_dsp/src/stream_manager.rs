use rx888_stream::device::{Rx888Device, StartResult};

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
    use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use num_complex::Complex32;

use crate::pipeline::FftPipeline;


/// Shared statistics between threads
pub struct StreamStats {
    pub bytes_in: AtomicU64,
    pub frames_out: AtomicU64,
}

/// Main stream manager: owns producer + consumer threads
pub struct StreamManager {
    running: Arc<AtomicBool>,
    spec_rx: Receiver<Vec<Complex32>>,
    stats: Arc<StreamStats>,
}

impl StreamManager {
    pub fn start(n: usize, v: usize, chunk_samples: usize) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let stats = Arc::new(StreamStats {
            bytes_in: AtomicU64::new(0),
            frames_out: AtomicU64::new(0),
        });

        let (sample_tx, sample_rx) = bounded::<Vec<u8>>(8);
        let (spec_tx, spec_rx) = bounded::<Vec<Complex32>>(32);

        // ───────────────────────────────────────────────────────────────
        // PRODUCER THREAD
        // ───────────────────────────────────────────────────────────────
        {
            let running = running.clone();
            let stats = stats.clone();

            thread::spawn(move || {
                // 1. Open the RX‑888 device
                let mut dev = match Rx888Device::open(0) {
                    Ok(d) => {
                        println!("RX-888 opened successfully!");
                        d
                    }
                    Err(e) => {
                        eprintln!("Failed to open RX-888: {:?}", e);
                        running.store(false, Ordering::Relaxed);
                        return;
                    }
                };

                // 2. Start streaming (bootloader/runtime detection)
                match dev.start_stream(8_000_000) {
                    Ok(result) => match result {
                        StartResult::BootloaderUploaded => {
                            eprintln!("Firmware uploaded — waiting for device to re-enumerate...");

                            // Drop old handle before re-enumeration
                            drop(dev);

                            // Initial wait to let Windows detach old handle
                            std::thread::sleep(std::time::Duration::from_millis(800));

                            // Robust re-open loop (Windows FX3 quirk)
                            let mut reopened: Option<Rx888Device> = None;

                            for attempt in 0..15 {
                                match Rx888Device::open(0) {
                                    Ok(d2) => {
                                        println!(
                                            "Re-opened runtime device successfully on attempt {}",
                                            attempt + 1
                                        );
                                        reopened = Some(d2);
                                        break;
                                    }
                                    Err(e) => {
                                        println!(
                                            "Re-open attempt {} failed: {:?} — retrying...",
                                            attempt + 1,
                                            e
                                        );
                                        std::thread::sleep(std::time::Duration::from_millis(200));
                                    }
                                }
                            }

                            let mut dev = match reopened {
                                Some(d2) => d2,
                                None => {
                                    eprintln!(
                                        "Failed to re-open runtime device after firmware upload"
                                    );
                                    running.store(false, Ordering::Relaxed);
                                    return;
                                }
                            };

                            // Now in runtime mode: start ADC + FX3 stream
                            if let Err(e) = dev.start_runtime_stream(80_000_000) {
                                eprintln!("Failed to start runtime stream: {:?}", e);
                                running.store(false, Ordering::Relaxed);
                                return;
                            }

                            // 3. Allocate reusable buffer
                            let mut buf = vec![0u8; chunk_samples * 2];

                            // 4. Main producer loop
                            while running.load(Ordering::Relaxed) {
                                if let Err(e) = dev.read_samples(&mut buf) {
                                    eprintln!("RX‑888 read error: {:?}", e);
                                    running.store(false, Ordering::Relaxed);
                                    break;
                                }

                                stats
                                    .bytes_in
                                    .fetch_add(buf.len() as u64, Ordering::Relaxed);

                                if sample_tx.send(buf.clone()).is_err() {
                                    break;
                                }
                            }
                        }

                        StartResult::AlreadyRuntime => {
                            eprintln!("Device already in runtime mode — continuing");

                            // Already in 0x00f1, just kick ADC + stream
                            if let Err(e) = dev.start_runtime_stream(80_000_000) {
                                eprintln!("Failed to start runtime stream: {:?}", e);
                                running.store(false, Ordering::Relaxed);
                                return;
                            }

                            // 3. Allocate reusable buffer
                            let mut buf = vec![0u8; chunk_samples * 2];

                            // 4. Main producer loop
                            while running.load(Ordering::Relaxed) {
                                if let Err(e) = dev.read_samples(&mut buf) {
                                    eprintln!("RX‑888 read error: {:?}", e);
                                    running.store(false, Ordering::Relaxed);
                                    break;
                                }

                                stats
                                    .bytes_in
                                    .fetch_add(buf.len() as u64, Ordering::Relaxed);

                                if sample_tx.send(buf.clone()).is_err() {
                                    break;
                                }
                            }
                        }
                    },

                    Err(e) => {
                        eprintln!("start_stream failed: {:?}", e);
                        running.store(false, Ordering::Relaxed);
                        return;
                    }
                }
            });
        }

        // ───────────────────────────────────────────────────────────────
        // CONSUMER THREAD (FFT pipeline)
        // ───────────────────────────────────────────────────────────────
        {
            let running = running.clone();
            let stats = stats.clone();

            thread::spawn(move || {
                let mut fft = FftPipeline::new(n, v);
                let mut out_frames = Vec::new();

                while running.load(Ordering::Relaxed) {
                    let chunk = match sample_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    out_frames.clear();
                    fft.push_bytes(&chunk, &mut out_frames);

                    for spec in out_frames.drain(..) {
                        stats.frames_out.fetch_add(1, Ordering::Relaxed);

                        if spec_tx.send(spec).is_err() {
                            running.store(false, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });
        }

        Self {
            running,
            spec_rx,
            stats,
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn try_recv_frame(&self) -> Option<Vec<Complex32>> {
        self.spec_rx.try_recv().ok()
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.stats.bytes_in.load(Ordering::Relaxed),
            self.stats.frames_out.load(Ordering::Relaxed),
        )
    }
}