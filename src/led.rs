use std::io::Write;
use crate::{Controller};
use color_eyre::{Result};

/// Controller led management.
pub struct Led<'a> {
    controller: &'a mut Controller<>,
}

impl<'a, 'b> Led<'a> {
    #[doc(hidden)]
    pub fn new(controller: &'a mut Controller<>) -> Led<'a> {
        Led {
            controller,
        }
    }

    /// Change the LED luminosity.
    pub fn level(self, value: u8) -> Result<()> {
        self.controller.control_with(0x87, 0x03, |mut buf| {
            buf.write(&[0x2d, value])
        })
    }

    /// Turn the LED off.
    pub fn off(self) -> Result<()> {
        self.level(0)
    }

    /// Turn the LED on.
    pub fn on(self) -> Result<()> {
        self.level(100)
    }
}
