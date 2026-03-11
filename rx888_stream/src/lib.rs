// rx888_stream/src/lib.rs

pub mod device;
pub mod init;
pub mod stream_manager;

pub use crate::device::{Rx888Device, Rx888Error, Rx888Result, DeviceMode};
pub use crate::init::SampleRate;
pub use crate::stream_manager::StreamManager;
