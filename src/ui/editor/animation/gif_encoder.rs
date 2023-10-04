use std::{fs::File, path::Path, sync::mpsc::Sender};

use crate::TerminalResult;

use super::encoding::AnimationEncoder;

pub struct GifEncoder {}

impl AnimationEncoder for GifEncoder {
    fn label(&self) -> String {
        "GIF".to_string()
    }
    fn extension(&self) -> String {
        "gif".to_string()
    }

    fn encode(&self, path: &Path, frames: Vec<(Vec<u8>, u32)>, width: usize, height: usize, sender: Sender<usize>) -> TerminalResult<()> {
        let mut image = File::create(path)?;
        let width = width as u16;
        let height = height as u16;

        let Ok(mut encoder) = ::gif::Encoder::new(&mut image, width, height, &[]) else {
            return Err(anyhow::anyhow!("Could not create encoder"));
        };
        encoder.set_repeat(::gif::Repeat::Infinite).unwrap();

        for (i, (mut data, _)) in frames.into_iter().enumerate() {
            sender.send(i)?;
            let gif_frame = ::gif::Frame::from_rgba(width, height, &mut data);
            encoder.write_frame(&gif_frame)?;
        }
        Ok(())
    }
}
