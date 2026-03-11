// rx888_stream/src/init.rs

use crate::device::{Rx888Device, Rx888Result};
use crate::device::ArgumentList;

#[derive(Debug, Clone, Copy)]
pub enum SampleRate {
    Sps32_4M,
    Sps64_8M,
    Sps129_6M,
}

fn adc_freq_for(rate: SampleRate) -> u32 {
    match rate {
        SampleRate::Sps32_4M => 32_400_000,
        SampleRate::Sps64_8M => 64_800_000,
        SampleRate::Sps129_6M => 129_600_000,
    }
}

pub fn initialize_device(dev: &mut Rx888Device, rate: SampleRate) -> Rx888Result<()> {
    
    let adc_freq = adc_freq_for(rate);

    // HF mode: do NOT initialize tuner
    // dev.tuner_init()?;   <-- REMOVE THIS
    
    eprintln!("initialize_device() → sending SETARG commands");
    
    // Optional: set HF gains/attenuators
    dev.send_argument(ArgumentList::DAT31_ATT, 0)?;     // 0 dB attenuation
    dev.send_argument(ArgumentList::AD8340_VGA, 10)?;   // example gain
    dev.send_argument(ArgumentList::PRESELECTOR, 0)?;   // wideband

    // Start ADC
    dev.start_adc(adc_freq)?;

    Ok(())
}


// pub fn initialize_device(dev: &mut Rx888Device, rate: SampleRate) -> Rx888Result<()> {
//     let adc_freq = adc_freq_for(rate);

//     dev.init_hf()?;
    
//     // Start ADC clock
//     dev.start_adc(adc_freq)?;

//     // Basic tuner bring‑up
//     dev.tuner_init()?;

//     Ok(())
// }