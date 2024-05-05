use crate::{Controller};
use color_eyre::{Result};

/// Calibration manager.
pub struct Calibrate<'a> {
    controller: &'a mut Controller<>,
}

impl<'a, 'b> Calibrate<'a> {
    #[doc(hidden)]
    pub fn new(controller: &'a mut Controller<>) -> Calibrate<'a> {
        Calibrate {
            controller,
        }
    }

    /// Calibrate the trackpads.
    pub fn trackpad(self) -> Result<()> {
        self.controller.control(0xa7)
    }

    /// Calibrate the joystick.
    pub fn joystick(self) -> Result<()> {
        self.controller.control(0xbf)
    }

    /// Calibrate the sensors.
    pub fn sensors(self) -> Result<()> {
        self.controller.control(0xb5)
    }
}
