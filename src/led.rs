use std::io::Write;
use {Result as Res, Controller};

/// Controller led management.
pub struct Led<'a> {
	controller: &'a mut Controller<>,
}

impl<'a, 'b> Led<'a> {
	#[doc(hidden)]
	pub fn new(controller: &'a mut Controller<>) -> Led<'a> {
		Led {
			controller: controller,
		}
	}

	/// Change the led luminosity.
	pub fn level(self, value: u8) -> Res<()> {
		self.controller.control_with(0x87, 0x03, |mut buf| {
			buf.write(&[0x2d, value])
		})
	}

	/// Turn the led off.
	pub fn off(self) -> Res<()> {
		self.level(0)
	}

	/// Turn the led on.
	pub fn on(self) -> Res<()> {
		self.level(100)
	}
}