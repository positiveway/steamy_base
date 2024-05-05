use crate::{Controller};
use color_eyre::{Result};

/// Controller led management.
pub struct Lizard<'a> {
    controller: &'a mut Controller,
}

impl<'a, 'b> Lizard<'a> {
    #[doc(hidden)]
    pub fn new(controller: &'a mut Controller) -> Lizard<'a> {
        Lizard {
            controller,
        }
    }

    /// Enable lizard mode.
    pub fn enable(self) -> Result<()> {
        self.controller.settings().lizard = true;
        self.controller.reset()
    }

    /// Disable lizard mode.
    pub fn disable(self) -> Result<()> {
        self.controller.settings().lizard = false;
        self.controller.reset()
    }
}
