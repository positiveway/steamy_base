use std::io::Write;
use std::time::Duration;
use byteorder::{WriteBytesExt, LittleEndian};
use crate::{Controller};
use color_eyre::{Result};

const RATIO: f64 = 495483.0;
const FREQUENCIES: [f64; 128] = [8.1758, 8.66196, 9.17702, 9.72272, 10.3009, 10.9134, 11.5623, 12.2499, 12.9783, 13.75, 14.5676, 15.4339, 16.3516, 17.3239, 18.354, 19.4454, 20.6017, 21.8268, 23.1247, 24.4997, 25.9565, 27.5, 29.1352, 30.8677, 32.7032, 34.6478, 36.7081, 38.8909, 41.2034, 43.6535, 46.2493, 48.9994, 51.9131, 55.0, 58.2705, 61.7354, 65.4064, 69.2957, 73.4162, 77.7817, 82.4069, 87.3071, 92.4986, 97.9989, 103.826, 110.0, 116.541, 123.471, 130.813, 138.591, 146.832, 155.563, 164.814, 174.614, 184.997, 195.998, 207.652, 220.0, 233.082, 246.942, 261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994, 391.995, 415.305, 440.0, 466.164, 493.883, 523.251, 554.365, 587.33, 622.254, 659.255, 698.456, 739.989, 783.991, 830.609, 880.0, 932.328, 987.767, 1046.5, 1108.73, 1174.66, 1244.51, 1318.51, 1396.91, 1479.98, 1567.98, 1661.22, 1760.0, 1864.66, 1975.53, 2093.0, 2217.46, 2349.32, 2489.02, 2637.02, 2793.83, 2959.96, 3135.96, 3322.44, 3520.0, 3729.31, 3951.07, 4186.01, 4434.92, 4698.64, 4978.03, 5274.04, 5587.65, 5919.91, 6271.93, 6644.88, 7040.0, 7458.62, 7902.13, 8372.02, 8869.84, 9397.27, 9956.06, 10548.1, 11175.3, 11839.8, 12543.9];

/// Representation of a note.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

pub struct Sound<'a> {
    controller: &'a mut Controller<>,
    channel: u8,
    note: Note,
    sharp: bool,
    octave: u8,
    duration: f64,
}

impl<'a, 'b> Sound<'a> {
    #[doc(hidden)]
    pub fn new(controller: &'a mut Controller<>) -> Sound<'a> {
        Sound {
            controller: controller,
            channel: 0,
            note: Note::C,
            sharp: false,
            octave: 6,
            duration: -1.0,
        }
    }

    /// Test a notification sound.
    pub fn test(self, value: u8) -> Result<()> {
        self.controller.control_with(0xb6, 0x04, |mut buf| {
            buf.write_u8(value)
        })
    }

    /// Change the notification sound when turning on and off the device.
    pub fn notification(self, on: u8, off: u8) -> Result<()> {
        self.controller.control_with(0xc1, 0x10, |mut buf| {
            buf.write_u8(on)?;
            buf.write_u8(off)?;

            buf.write(&[
                0xff, 0xff, 0x03, 0x09,
                0x05, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ][..])
        })
    }

    /// Send the sound on the left channel.
    pub fn left(mut self) -> Self {
        self.channel = 1;
        self
    }

    /// Send the sound on the right channel.
    pub fn right(mut self) -> Self {
        self.channel = 0;
        self
    }

    /// The note to reproduce.
    pub fn note(mut self, value: Note) -> Self {
        self.note = value;
        self
    }

    /// Whether it's a sharp.
    pub fn sharp(mut self) -> Self {
        self.sharp = true;
        self
    }

    /// The octave.
    pub fn octave(mut self, value: u8) -> Self {
        self.octave = value;
        self
    }

    /// The duration of the note.
    pub fn duration(mut self, value: Duration) -> Self {
        self.duration = value.as_secs() as f64 + (value.subsec_nanos() as f64 / 1_000_000_000.0);
        self
    }

    /// Play the note.
    pub fn play(self) -> Result<()> {
        let index = match self.note {
            Note::C => if self.sharp { 1 } else { 0 },
            Note::D => if self.sharp { 3 } else { 2 },
            Note::E => 4,
            Note::F => if self.sharp { 6 } else { 5 },
            Note::G => if self.sharp { 8 } else { 7 },
            Note::A => if self.sharp { 10 } else { 9 },
            Note::B => 11,
        } + (self.octave * 12) as usize;

        let channel = self.channel;
        let duration = self.duration;
        let period = 1.0 / FREQUENCIES[if index >= 128 { 127 } else { index }];

        self.controller.control_with(0x8f, 0x07, |mut buf| {
            buf.write_u8(channel)?;

            buf.write_u16::<LittleEndian>((period * RATIO).round() as u16)?;
            buf.write_u16::<LittleEndian>((period * RATIO).round() as u16)?;

            if duration >= 0.0 {
                buf.write_u16::<LittleEndian>((duration / period).round() as u16)?;
            } else {
                buf.write_u16::<LittleEndian>(0x7fff)?;
            }

            Ok(())
        })
    }

    /// Stop playing.
    pub fn stop(self) -> Result<()> {
        let channel = self.channel;

        self.controller.control_with(0x8f, 0x07, |mut buf| {
            buf.write_u8(channel)
        })
    }
}
