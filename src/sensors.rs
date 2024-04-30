use crate::{Result as Res, Controller};

/// Controller sensors management.
pub struct Sensors<'a> {
	controller: &'a mut Controller<>,
}

impl<'a, 'b> Sensors<'a> {
	#[doc(hidden)]
	pub fn new(controller: &'a mut Controller<>) -> Sensors<'a> {
		Sensors {
			controller,
		}
	}

	/// Turn the sensors off.
	pub fn off(self) -> Res<()> {
		self.controller.settings().sensors = false;
		self.controller.reset()
	}

	/// Turn the sensors on.
	pub fn on(self) -> Res<()> {
		self.controller.settings().sensors = true;
		self.controller.reset()
	}
}
