use byteorder::{WriteBytesExt, LittleEndian};
use {Result as Res, Controller};

/// Controller feedback builder.
pub struct Feedback<'a> {
	controller: &'a mut Controller<>,
	side:       u8,
	amplitude:  u16,
	period:     u16,
	count:      u16,
}

impl<'a> Feedback<'a> {
	#[doc(hidden)]
	pub fn new(controller: &'a mut Controller<>) -> Feedback<'a> {
		Feedback {
			controller,
			side:       0,
			amplitude:  128,
			period:     0,
			count:      1,
		}
	}

	/// Send the feedback on the left pad.
	pub fn left(mut self) -> Self {
		self.side = 1;
		self
	}

	/// Send the feedback on the right pad.
	pub fn right(mut self) -> Self {
		self.side = 0;
		self
	}

	/// The amplitude of the feedback.
	pub fn amplitude(mut self, value: u16) -> Self {
		self.amplitude = value;
		self
	}

	/// The period of the feedback.
	pub fn period(mut self, value: u16) -> Self {
		self.period = value;
		self
	}

	/// The number of feedbacks to send.
	pub fn count(mut self, value: u16) -> Self {
		self.count = value;
		self
	}

	/// Send the built feedback.
	pub fn send(self) -> Res<()> {
		let side      = self.side;
		let amplitude = self.amplitude;
		let period    = self.period;
		let count     = self.count;

		self.controller.control_with(0x8f, 0x08, |mut buf| {
			buf.write_u8(side)?;
			buf.write_u16::<LittleEndian>(amplitude)?;
			buf.write_u16::<LittleEndian>(period)?;
			buf.write_u16::<LittleEndian>(count)?;

			Ok(())
		})
	}
}
