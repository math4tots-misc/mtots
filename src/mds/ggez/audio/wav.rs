use super::SoundData;
use crate::mtry;
use crate::Result;
use std::f32::consts::PI;
use std::io::Cursor;

impl SoundData {
    pub fn from_samples(data: &[i16]) -> Result<Self> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut bytes = Cursor::new(Vec::<u8>::new());
        {
            let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
            for sample in data {
                mtry!(writer.write_sample(*sample));
            }
            mtry!(writer.finalize());
        }
        Ok(Self::from_bytes(&bytes.into_inner()))
    }
    pub fn sine(nsamples: usize, hertz: u16, amp: f32) -> Result<Self> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut bytes = Cursor::new(Vec::<u8>::new());
        {
            let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
            for t in 0..nsamples {
                let t = t as f32 / 44100.0;
                let sample = (t * (hertz as f32) * 2.0 * PI).sin();
                writer.write_sample((sample * amp) as i16).unwrap();
            }
            mtry!(writer.finalize());
        }
        Ok(Self::from_bytes(&bytes.into_inner()))
    }
}
