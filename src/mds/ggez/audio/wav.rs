use crate::mtry;
use crate::Result;
use super::SoundData;
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
}
