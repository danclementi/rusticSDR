#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

const VID: u16 = 0x04B4;
const PID_BOOT: u16 = 0x00F3;
const PID_RUNTIME: u16 = 0x00F1;

use std::io::{Error as IoError, ErrorKind};

use std::io::{self, Read, Cursor};
use std::num::Wrapping;
use std::time::Duration;
// use anyhow::Result;


use rusb::constants::{
    LIBUSB_ENDPOINT_OUT,
    LIBUSB_ENDPOINT_IN,
    LIBUSB_REQUEST_TYPE_VENDOR,
};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum FX3Command {
    STARTFX3   = 0xAA,
    STOPFX3    = 0xAB,
    TESTFX3    = 0xAC,
    GPIOFX3    = 0xAD,
    I2CWFX3    = 0xAE,
    I2CRFX3    = 0xAF,
    RESETFX3   = 0xB1,
    SETARGFX3  = 0xB6,
    STARTADC   = 0xB2,
    TUNERINIT  = 0xB4,
    TUNERTUNE  = 0xB5,
    TUNERSTDBY = 0xB8,
    READINFODEBUG = 0xBA,
}

#[derive(Copy, Clone, Debug)]
pub enum ArgumentList {
    R82XX_ATTENUATOR = 1,
    R82XX_VGA = 2,
    R82XX_SIDEBAND = 3,
    R82XX_HARMONIC = 4,
    DAT31_ATT = 10,
    AD8340_VGA = 11,
    PRESELECTOR = 12,
    VHF_ATTENUATOR = 13,
}

#[derive(Copy, Clone, Debug)]
pub enum GPIOPin {
    ATT_LE = 1 << 0,
    ATT_CLK = 1 << 1,
    ATT_DATA = 1 << 2,
    SEL0 = 1 << 3,
    SEL1 = 1 << 4,
    SHDWN = 1 << 5,
    DITH = 1 << 6,
    RANDO = 1 << 7,
    BIAS_HF = 1 << 8,
    BIAS_VHF = 1 << 9,
    LED_YELLOW = 1 << 10,
    LED_RED = 1 << 11,
    LED_BLUE = 1 << 12,
    ATT_SEL0 = 1 << 13,
    ATT_SEL1 = 1 << 14,
    VHF_EN = 1 << 15,
    PGA_EN = 1 << 16,
}

pub type Rx888Result<T> = Result<T, Rx888Error>;

#[derive(Debug)]
pub enum StartResult {
    BootloaderUploaded,
    AlreadyRuntime,
}

// #[derive(Debug, Clone, Copy)]
// pub enum DeviceMode {
//     Bootloader,
//     Runtime,
// }

#[derive(Debug)]
pub enum Rx888Error {
    Io(IoError),
    Usb(String),
    Firmware(String),
    DeviceNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    Bootloader,
    Runtime,
}

pub struct Rx888Device {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    mode: DeviceMode,
}

// Path is relative to src/device.rs
const FX3_FIRMWARE: &[u8] = include_bytes!("../../rx888_stream/SDDC_FX3.img");


impl From<IoError> for Rx888Error {
    fn from(e: IoError) -> Self {
        Rx888Error::Io(e)
    }
}

impl std::fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceMode::Bootloader => write!(f, "bootloader"),
            DeviceMode::Runtime => write!(f, "runtime"),
        }
    }
}

impl Rx888Device {

    pub fn mode(&self) -> DeviceMode {
        self.mode
    }

    fn send_cmd_u32(&mut self, cmd: FX3Command, data: u32) -> Rx888Result<()> {
        let timeout = Duration::from_secs(1);
        self.handle
            .write_control(
                LIBUSB_ENDPOINT_OUT | LIBUSB_REQUEST_TYPE_VENDOR,
                cmd as u8,
                0,
                0,
                &data.to_le_bytes(),
                timeout,
            )
            .map_err(|e| Rx888Error::Usb(format!("FX3 cmd {:?} failed: {e}", cmd as u8)))?;
        Ok(())
    }

    fn send_cmd_u64(&mut self, cmd: FX3Command, data: u64) -> Rx888Result<()> {
        let timeout = Duration::from_secs(1);
        self.handle
            .write_control(
                LIBUSB_ENDPOINT_OUT | LIBUSB_REQUEST_TYPE_VENDOR,
                cmd as u8,
                0,
                0,
                &data.to_le_bytes(),
                timeout,
            )
            .map_err(|e| Rx888Error::Usb(format!("FX3 cmd {:?} failed: {e}", cmd as u8)))?;
        Ok(())
    }

    pub fn send_argument(&mut self, arg: ArgumentList, value: u16) -> Rx888Result<()> {
        let timeout = Duration::from_secs(1);
        self.handle.write_control(
            LIBUSB_ENDPOINT_OUT | LIBUSB_REQUEST_TYPE_VENDOR,
            FX3Command::SETARGFX3 as u8,
            value,
            arg as u16,
            &[0],
            timeout,
        ).map_err(|e| Rx888Error::Usb(format!("SETARG {:?} failed: {e}", arg)))?;
        Ok(())
    }

    /// Minimal bring‑up: start ADC clock + start FX3 streaming
    pub fn start_runtime_stream(&mut self, adc_freq_hz: u32) -> Rx888Result<()> {
        // ADC clock (depends on your board; 80e6 is common for RX‑888 MkII)
        self.send_cmd_u32(FX3Command::STARTADC, adc_freq_hz)?;
        // Start FX3 GPII engine / data path
        self.send_cmd_u32(FX3Command::STARTFX3, 0)?;
        Ok(())
    }

    pub fn open(index: usize) -> Rx888Result<Self> {
        let devices = rusb::devices().map_err(|e| Rx888Error::Usb(e.to_string()))?;

        let mut runtime_matches = Vec::new();
        let mut boot_matches = Vec::new();

        eprintln!("entered open....");

        for device in devices.iter() {
            let desc = device.device_descriptor().map_err(|e| Rx888Error::Usb(e.to_string()))?;
            let vid = desc.vendor_id();
            let pid = desc.product_id();

            if vid == VID {
                eprintln!("RX-888 candidate: {:04x}:{:04x}", vid, pid);
            }

            if vid == VID && pid == PID_RUNTIME {
                runtime_matches.push(device.clone());
            }

            if vid == VID && pid == PID_BOOT {
                boot_matches.push(device.clone());
            }
        }

        // Prefer runtime firmware
        let target_list = if !runtime_matches.is_empty() {
            &runtime_matches
        } else if !boot_matches.is_empty() {
            &boot_matches
        } else {
            return Err(Rx888Error::DeviceNotFound);
        };

        if index >= target_list.len() {
            return Err(Rx888Error::DeviceNotFound);
        }

        let mut handle = target_list[index]
            .open()
            .map_err(|e| Rx888Error::Usb(e.to_string()))?;

        handle.claim_interface(0).map_err(|e| Rx888Error::Usb(e.to_string()))?;

        let mode = if !runtime_matches.is_empty() {
            DeviceMode::Runtime
        } else {
            DeviceMode::Bootloader
        };

        // ⭐ FIX: FX3 needs time to settle when already in runtime mode
        if mode == DeviceMode::Runtime {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        Ok(Self { handle, mode })
    }

    /// Blocking read of raw int16 samples into the provided buffer.
    ///
    /// `buf` must be sized to hold N * 2 bytes (for N int16 samples).
    pub fn read_samples(&mut self, buf: &mut [u8]) -> Rx888Result<()> {
        let timeout = std::time::Duration::from_millis(100);

        let n = self
            .handle
            .read_bulk(0x81, buf, timeout)
            .map_err(|e| Rx888Error::Usb(e.to_string()))?;

        // eprintln!("read_bulk returned {} bytes", n);

        if n == 0 {
            return Err(Rx888Error::Usb("Zero-length read".to_string()));
        }

        // Don’t treat short reads as fatal yet; just log them
        if n != buf.len() {
            eprintln!(
                "RX-888 short read: {} bytes (expected {})",
                n,
                buf.len()
            );
        }

        Ok(())
    }
    /// Write a single RX-888 Mk II register.
    pub fn write_reg(&self, addr: u8, value: u8) -> Rx888Result<()> {
        self.handle
            .write_control(
                0x40,
                0xB0,
                value as u16,
                addr as u16,
                &[],
                Duration::from_millis(10),
            )
            .map_err(|e| Rx888Error::Usb(format!("write_reg({addr:#04x}) failed: {e}")))?;
        Ok(())
    }

    /// Initialize the HF front-end path (0–32 MHz).
    /// This must be called AFTER firmware load and BEFORE ADC start.
    pub fn init_hf(&self) -> Rx888Result<()> {
        println!("Initializing HF is called");
        self.write_reg(0x01, 0x00)?; // HF path
        self.write_reg(0x02, 20)?;   // RF attenuator
        self.write_reg(0x03, 0)?;    // LNA gain
        self.write_reg(0x04, 0)?;    // IF gain
        self.write_reg(0x05, 0)?;    // ADC PGA
        self.write_reg(0x06, 0x00)?; // ADC mode
        self.write_reg(0x07, 0x01)?; // FPGA routing
        Ok(())
    }


    pub fn start_adc(&mut self, adc_freq_hz: u32) -> Rx888Result<()> {
        self.send_cmd_u32(FX3Command::STARTADC, adc_freq_hz)
    }

    pub fn start_fx3(&mut self) -> Rx888Result<()> {
        eprintln!("STARTFX3 → sending command");
        self.send_cmd_u32(FX3Command::STARTFX3, 0)
    }

    pub fn stop_fx3(&mut self) -> Rx888Result<()> {
        eprintln!("STOPFX3 → sending command");
        self.send_cmd_u32(FX3Command::STOPFX3, 0)
    }

    pub fn reset_fx3(&mut self) -> Rx888Result<()> {
        self.send_cmd_u32(FX3Command::RESETFX3, 0)
    }

    pub fn tuner_init(&mut self) -> Rx888Result<()> {
        self.send_cmd_u32(FX3Command::TUNERINIT, 0)
    }

    pub fn tuner_tune(&mut self, freq_hz: u64) -> Rx888Result<()> {
        self.send_cmd_u64(FX3Command::TUNERTUNE, freq_hz)
    }

    pub fn tuner_standby(&mut self) -> Rx888Result<()> {
        self.send_cmd_u32(FX3Command::TUNERSTDBY, 0)
    }

    pub fn set_gain(&mut self, arg: ArgumentList, value: u16) -> Rx888Result<()> {
        self.send_argument(arg, value)
    }

    pub fn gpio_write(&mut self, mask: u32) -> Rx888Result<()> {
        self.send_cmd_u32(FX3Command::GPIOFX3, mask)
    }

    pub fn read_debug(&mut self) -> Rx888Result<Vec<u8>> {
        let mut buf = vec![0u8; 512];
        let timeout = Duration::from_secs(1);

        let n = self.handle.read_control(
            LIBUSB_ENDPOINT_IN | LIBUSB_REQUEST_TYPE_VENDOR,
            FX3Command::READINFODEBUG as u8,
            0,
            0,
            &mut buf,
            timeout,
        ).map_err(|e| Rx888Error::Usb(format!("READINFODEBUG failed: {e}")))?;

        buf.truncate(n);
        Ok(buf)
    }

    pub fn set_gain_from_python(&mut self, arg: u16, value: u16) -> Rx888Result<()> {
        let arg_enum = match arg {
            1 => ArgumentList::R82XX_ATTENUATOR,
            2 => ArgumentList::R82XX_VGA,
            3 => ArgumentList::R82XX_SIDEBAND,
            4 => ArgumentList::R82XX_HARMONIC,
            10 => ArgumentList::DAT31_ATT,
            11 => ArgumentList::AD8340_VGA,
            12 => ArgumentList::PRESELECTOR,
            13 => ArgumentList::VHF_ATTENUATOR,
            _ => return Err(Rx888Error::Usb("Invalid argument ID".into())),
        };

        self.set_gain(arg_enum, value)
    }

    pub fn download_fx3_firmware(&mut self) -> Rx888Result<()> {
        // FX3 .img loader, adapted from original fx3_load_ram
        const LIBUSB_ENDPOINT_OUT: u8 = 0x00;
        const LIBUSB_ENDPOINT_IN: u8 = 0x80;
        const LIBUSB_REQUEST_TYPE_VENDOR: u8 = 0x40;
        const LIBUSB_RECIPIENT_DEVICE: u8 = 0x00;
        const RW_INTERNAL: u8 = 0xA0;

        let timeout = Duration::from_secs(1);
        let mut ram = Cursor::new(FX3_FIRMWARE);

        // Header: "CY" ... 0xB0
        let mut header = [0u8; 4];
        ram.read_exact(&mut header)
            .map_err(|e| Rx888Error::Usb(format!("FX3 header read failed: {e}")))?;

        if header[0] != b'C' || header[1] != b'Y' {
            return Err(Rx888Error::Usb("Invalid FX3 image header".into()));
        }
        if header[3] != 0xB0 {
            return Err(Rx888Error::Usb("Unsupported FX3 image type".into()));
        }

        let mut checksum: Wrapping<u32> = Wrapping(0);

        // Main load loop
        let jump_address = loop {
            // length (in 32‑bit words)
            let length = {
                let mut buf = [0u8; 4];
                ram.read_exact(&mut buf)
                    .map_err(|e| Rx888Error::Usb(format!("FX3 length read failed: {e}")))?;
                u32::from_le_bytes(buf)
            };

            // address
            let address = {
                let mut buf = [0u8; 4];
                ram.read_exact(&mut buf)
                    .map_err(|e| Rx888Error::Usb(format!("FX3 address read failed: {e}")))?;
                u32::from_le_bytes(buf)
            };

            if length == 0 {
                // this address is the jump address
                break address;
            }

            let byte_len = (length as usize) * 4;
            let mut data = vec![0u8; byte_len];
            ram.read_exact(&mut data)
                .map_err(|e| Rx888Error::Usb(format!("FX3 data read failed: {e}")))?;

            // accumulate checksum
            checksum += data
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .map(Wrapping)
                .sum::<Wrapping<u32>>();

            // write + verify in 4096‑byte chunks
            for (offset, chunk) in data.chunks(4096).enumerate() {
                let addr = address + (offset as u32) * 4096;

                // write
                self.handle
                    .write_control(
                        LIBUSB_ENDPOINT_OUT
                            | LIBUSB_REQUEST_TYPE_VENDOR
                            | LIBUSB_RECIPIENT_DEVICE,
                        RW_INTERNAL,
                        (addr & 0xFFFF) as u16,
                        (addr >> 16) as u16,
                        chunk,
                        timeout,
                    )
                    .map_err(|e| Rx888Error::Usb(format!("FX3 write failed: {e}")))?;

                // read back
                let mut readback = [0u8; 4096];
                self.handle
                    .read_control(
                        LIBUSB_ENDPOINT_IN
                            | LIBUSB_REQUEST_TYPE_VENDOR
                            | LIBUSB_RECIPIENT_DEVICE,
                        RW_INTERNAL,
                        (addr & 0xFFFF) as u16,
                        (addr >> 16) as u16,
                        &mut readback,
                        timeout,
                    )
                    .map_err(|e| Rx888Error::Usb(format!("FX3 verify read failed: {e}")))?;

                if chunk != &readback[..chunk.len()] {
                    return Err(Rx888Error::Usb("FX3 verify mismatch".into()));
                }
            }
        };

        // checksum from image
        let firmware_checksum = {
            let mut buf = [0u8; 4];
            ram.read_exact(&mut buf)
                .map_err(|e| Rx888Error::Usb(format!("FX3 checksum read failed: {e}")))?;
            u32::from_le_bytes(buf)
        };

        if checksum.0 != firmware_checksum {
            return Err(Rx888Error::Usb(format!(
                "FX3 checksum mismatch: calc={:08x}, img={:08x}",
                checksum.0, firmware_checksum
            )));
        }

        // final jump
        self.handle
            .write_control(
                LIBUSB_ENDPOINT_OUT
                    | LIBUSB_REQUEST_TYPE_VENDOR
                    | LIBUSB_RECIPIENT_DEVICE,
                RW_INTERNAL,
                (jump_address & 0xFFFF) as u16,
                (jump_address >> 16) as u16,
                &[],
                timeout,
            )
            .map_err(|e| Rx888Error::Usb(format!("FX3 jump failed: {e}")))?;

        eprintln!("FX3 firmware download complete — device will re-enumerate");
        Ok(())
    }

    pub fn start_stream(&mut self, _rate: u32) -> Rx888Result<StartResult> {
        let desc = self.handle.device().device_descriptor()
            .map_err(|e| Rx888Error::Usb(e.to_string()))?;

        let pid = desc.product_id();

        // const PID_BOOT: u16 = 0x00F1;
        // const PID_RUNTIME: u16 = 0x00F3;

        if pid == PID_BOOT {
            eprintln!("start_stream: device in bootloader mode → uploading firmware");
            self.download_fx3_firmware()?;
            return Ok(StartResult::BootloaderUploaded);
        }

        if pid == PID_RUNTIME {
            eprintln!("start_stream: device already in runtime mode → skipping firmware upload");
            return Ok(StartResult::AlreadyRuntime);
        }

        Err(Rx888Error::Usb(format!("Unexpected PID {:04x}", pid)))
    }

}