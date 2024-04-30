use {Result as Res, Controller};

/// Calibration manager.
pub struct Calibrate<'a> {
	controller: &'a mut Controller<>,
}

impl<'a, 'b> Calibrate<'a> {
	#[doc(hidden)]
	pub fn new(controller: &'a mut Controller<>) -> Calibrate<'a> {
		Calibrate {
			controller: controller,
		}
	}

	/// Calibrate the trackpads.
	pub fn trackpad(self) -> Res<()> {
		self.controller.control(0xa7)
	}

	/// Calibrate the joystick.
	pub fn joystick(self) -> Res<()> {
		self.controller.control(0xbf)
	}

	/// Calibrate the sensors.
	pub fn sensors(self) -> Res<()> {
		self.controller.control(0xb5)
	}
}