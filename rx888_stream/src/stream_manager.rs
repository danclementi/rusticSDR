// rx888_stream/src/stream_manager.rs

use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::device::{DeviceMode, Rx888Device, Rx888Result};
use crate::init::{self, SampleRate};

pub struct StreamManager {
    dev: Rx888Device,
    rate: SampleRate,
    running: bool,
}

impl StreamManager {
    pub fn new(rate: SampleRate) -> Rx888Result<Self> {
        // First open attempt
        let mut dev = Rx888Device::open(0)?;

        // If in bootloader mode, upload FX3 firmware
        if matches!(dev.mode(), DeviceMode::Bootloader) {
            eprintln!("Device in bootloader mode → uploading FX3 firmware");
            dev.download_fx3_firmware()?;

            // Drop handle so the OS can re-enumerate
            drop(dev);

            // Wait for re-enumeration
            thread::sleep(Duration::from_secs(5));

            // Re-open as runtime
            dev = Rx888Device::open(0)?;
        }

        Ok(Self {
            dev,
            rate,
            running: false,
        })
    }

    pub fn start(&mut self) -> Rx888Result<()> {
        if self.running {
            return Ok(());
        }

        // Always stop FX3 first — safe even if not running
        let _ = self.dev.stop_fx3();
        let _ = self.dev.stop_fx3();

        // Give FX3 time to fully stop its streaming engine
        sleep(Duration::from_millis(500));

        // ADC + HF/VHF bring-up (uses SETARG)
        init::initialize_device(&mut self.dev, self.rate)?;

        // Now start FX3 streaming engine
        self.dev.start_fx3()?;

        self.running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Rx888Result<()> {
        if !self.running {
            return Ok(());
        }

        self.dev.stop_fx3()?;
        self.running = false;
        Ok(())
    }

    /// Blocking read of one chunk of IQ samples.
    pub fn read_samples(&mut self, buf: &mut [u8]) -> Rx888Result<()> {
        self.dev.read_samples(buf)
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.rate
    }
}