#[cfg(not(target_os = "linux"))]
use std::marker::PhantomData;

use std::time::Duration;
use std::thread;
use std::io::{self, Cursor, Write};
use byteorder::{WriteBytesExt, LittleEndian};

#[cfg(target_os = "linux")]
use usb;

#[cfg(not(target_os = "linux"))]
use hid;

use crate::{Result as Res, Error, State, Details};
use crate::{Lizard, Feedback, Sensors, Led, Sound, Calibrate, details};

const LIMIT:    u64 = 10;
const INCREASE: u64 = 50;

macro_rules! request {
	($limit:ident, $body:expr) => (
		match $body {
			Ok(v) => {
				v
			}

			Err(e) => {
				if $limit == 0 {
					Err(e)?;
				}

				thread::sleep(Duration::from_millis((LIMIT - $limit) * INCREASE));

				$limit -= 1;
				continue;
			}
		}
	)
}

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Settings {
	pub timeout: u16,
	pub sensors: bool,
	pub lizard:  bool,
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			timeout: 360,
			sensors: false,
			lizard:  false,
		}
	}
}

/// The controller.
#[cfg(target_os = "linux")]
pub struct Controller {
	handle:   usb::DeviceHandle<usb::GlobalContext>,
	packet:   [u8; 64],
	settings: Settings,

	product: u16,
	address: u8,
	index:   u16,
}

#[cfg(not(target_os = "linux"))]
pub struct Controller<'a> {
	handle:  hid::Handle,
	packet:  [u8; 65],
	settings: Settings,

	product: u16,
	marker:  PhantomData<&'a ()>,
}

impl<'a> Controller{
	#[doc(hidden)]
	#[cfg(target_os = "linux")]
	pub fn new(mut device: usb::Device<usb::GlobalContext>, mut handle: usb::DeviceHandle<usb::GlobalContext>, product: u16, endpoint: u8, index: u16) -> Res<Controller<>> {
		let mut address: Option<u8> = None;

		for i in 0 .. device.device_descriptor()?.num_configurations() {
			for interface in device.config_descriptor(i)?.interfaces() {
				if handle.kernel_driver_active(interface.number())? {
					handle.detach_kernel_driver(interface.number())?;
				}

				for descriptor in interface.descriptors() {
					if descriptor.class_code() == 3 &&
					   descriptor.sub_class_code() == 0 &&
					   descriptor.protocol_code() == 0
					{
						handle.claim_interface(descriptor.interface_number())?;
					}

					for end in descriptor.endpoint_descriptors() {
						if end.number() == endpoint {
							address = Some(end.address());
						}
					}
				}
			}
		}

		let mut controller = Controller {
			handle,
			packet:   [0u8; 64],
			settings: Default::default(),

			product,
			address: address.ok_or(usb::Error::InvalidParam)?,
			index,
		};

		controller.reset()?;
		// controller.led().off()?;
		// controller.sensors().off()?;
		// controller.timeout(Duration::from_secs((u16::MAX - 100) as u64))?;
		// controller.lizard().disable()?;
		// controller.sound().notification(0, 0)?;

		Ok(controller)
	}

	#[cfg(not(target_os = "linux"))]
	pub fn new<'b>(handle: hid::Handle, product: u16) -> Res<Controller<'b>> {
		let mut controller = Controller {
			handle:   handle,
			packet:   [0u8; 65],
			settings: Default::default(),

			product: product,

			marker: PhantomData,
		};

		controller.reset()?;

		Ok(controller)
	}

	/// Check if the controller is remote.
	pub fn is_remote(&self) -> bool {
		self.product == 0x1142
	}

	/// Check if the controller is wired.
	pub fn is_wired(&self) -> bool {
		self.product == 0x1102
	}

	/// Check if the controller is connected.
	pub fn is_connected(&mut self) -> bool {
		if self.is_wired() {
			return true;
		}

		if let Ok(buf) = self.request(0xb4) {
			buf[0] == 0x02
		}
		else {
			false
		}
	}

	#[doc(hidden)]
	pub fn settings(&mut self) -> &mut Settings {
		&mut self.settings
	}

	#[doc(hidden)]
	pub fn reset(&mut self) -> Res<()> {
		let timeout = self.settings.timeout;

		if self.settings.lizard {
			self.control(0x85)?;
		}
		else {
			self.control(0x81)?;
		}

		if self.settings.sensors {
			self.control_with(0x87, 0x15, |mut buf| {
				buf.write_u8(0x32)?;
				buf.write_u16::<LittleEndian>(timeout)?;
				buf.write(&[0x18, 0x00, 0x00, 0x31, 0x02, 0x00, 0x08, 0x07, 0x00, 0x07, 0x07, 0x00, 0x30])?;
				buf.write_u8(0x14)?;
				buf.write(&[0x00, 0x2e])
			})?;
		}
		else {
			self.control_with(0x87, 0x15, |mut buf| {
				buf.write_u8(0x32)?;
				buf.write_u16::<LittleEndian>(timeout)?;
				buf.write(&[0x18, 0x00, 0x00, 0x31, 0x02, 0x00, 0x08, 0x07, 0x00, 0x07, 0x07, 0x00, 0x30])?;
				buf.write_u8(0x00)?;
				buf.write(&[0x00, 0x2e])
			})?;
		}

		Ok(())
	}

	#[doc(hidden)]
	pub fn control(&mut self, id: u8) -> Res<()> {
		self.control_with(id, 0x00, |_| { Ok(()) })
	}

	#[doc(hidden)]
	#[cfg(target_os = "linux")]
	pub fn control_with<T, F>(&mut self, id: u8, size: u8, func: F) -> Res<()>
		where F: FnOnce(Cursor<&mut [u8]>) -> io::Result<T>
	{
		self.packet.clone_from_slice(&[0; 64][..]);
		self.packet[0] = id;
		self.packet[1] = size;

		func(Cursor::new(&mut self.packet[2..]))?;
		self.handle.write_control(0x21, 0x09, 0x0300, self.index, &self.packet[..], Duration::from_secs(0))?;

		Ok(())
	}

	#[doc(hidden)]
	#[cfg(not(target_os = "linux"))]
	pub fn control_with<T, F>(&mut self, func: F) -> Res<()>
		where F: FnOnce(Cursor<&mut [u8]>) -> io::Result<T>
	{
		self.packet.clone_from_slice(&[0; 65][..]);
		self.packet[1] = id;
		self.packet[2] = size;

		func(Cursor::new(&mut self.packet[3..]))?;
		self.handle.feature().send(&self.packet[..])?;

		Ok(())
	}

	#[doc(hidden)]
	pub fn request(&mut self, id: u8) -> Res<&[u8]> {
		self.request_with(id, 0x00, |_| Ok(()))
	}

	#[doc(hidden)]
	#[cfg(target_os = "linux")]
	pub fn request_with<T, F>(&mut self, id: u8, size: u8, func: F) -> Res<&[u8]>
		where F: FnOnce(Cursor<&mut [u8]>) -> io::Result<T>
	{
		self.packet.clone_from_slice(&[0; 64][..]);
		self.packet[0] = id;
		self.packet[1] = size;

		func(Cursor::new(&mut self.packet[2..]))?;

		let mut limit = LIMIT;
		loop {
			request!(limit, self.handle.write_control(0x21, 0x09, 0x0300, self.index,
				&self.packet[..], Duration::from_secs(0)));

			request!(limit, self.handle.read_control(0xa1, 0x01, 0x0300,
				self.index, &mut self.packet[..], Duration::from_secs(0)));

			if self.packet[0] == id && self.packet[1] != 0 {
				break;
			}

			request!(limit, Err(Error::NotSupported));
		}

		Ok(&self.packet[2 .. (self.packet[1] + 2) as usize])
	}

	#[doc(hidden)]
	#[cfg(not(target_os = "linux"))]
	pub fn request_with<T, F>(&mut self, id: u8, size: u8, func: F) -> Res<&[u8]>
		where F: FnOnce(Cursor<&mut [u8]>) -> io::Result<T>
	{
		self.packet.clone_from_slice(&[0; 65][..]);
		self.packet[1] = id;
		self.packet[2] = size;

		func(Cursor::new(&mut self.packet[3..]))?;

		let mut limit = LIMIT;
		loop {
			request!(limit, self.handle.feature().send(&self.packet[..]));
			request!(limit, self.handle.feature().get(&mut self.packet[..]));

			if self.packet[1] == id && self.packet[2] != 0 {
				break;
			}

			request!(limit, Err(Error::NotSupported));
		}

		Ok(&self.packet[3 .. (self.packet[2] + 2) as usize])
	}

	/// Get the lizard manager.
	pub fn lizard<'b>(&'b mut self) -> Lizard<'b> where 'a: 'b {
		Lizard::new(self)
	}

	/// Get the LED manager.
	pub fn led<'b>(&'b mut self) -> Led<'b> where 'a: 'b {
		Led::new(self)
	}

	/// Get the feedback builder.
	pub fn feedback<'b>(&'b mut self) -> Feedback<'b> where 'a: 'b {
		Feedback::new(self)
	}

	/// Get the sensor manager.
	pub fn sensors<'b>(&'b mut self) -> Sensors<'b> where 'a: 'b {
		Sensors::new(self)
	}

	/// Get the calibration manager.
	pub fn calibrate<'b>(&'b mut self) -> Calibrate<'b> where 'a: 'b {
		Calibrate::new(self)
	}

	/// Get the sound player.
	pub fn sound<'b>(&'b mut self) -> Sound<'b> where 'a: 'b {
		Sound::new(self)
	}

	/// Set the idle duration before turning off.
	pub fn timeout(&mut self, value: Duration) -> Res<()> {
		self.settings.timeout = value.as_secs() as u16;
		self.reset()
	}

	/// Turn the controller off.
	pub fn off(&mut self) -> Res<()> {
		self.control_with(0x9f, 0x04, |mut buf| {
			buf.write(b"off!")
		})
	}

	/// Fetch the controller details.
	pub fn details(&mut self) -> Res<Details> {
		if self.is_wired() {
			self.request(0x83)?;
		}

		let build = details::Build::parse(Cursor::new(self.request(0x83)?))?;

		let mainboard = details::Serial::parse(Cursor::new(self.request_with(0xae, 0x15, |mut buf| buf.write_u8(0x00))?))?;

		let controller = details::Serial::parse(Cursor::new(self.request_with(0xae, 0x15, |mut buf| buf.write_u8(0x01))?))?;

		let receiver = if self.is_remote() {
			Some(details::Receiver::parse(Cursor::new(self.request(0xa1)?))?)
		}
		else {
			None
		};

		Ok(Details {
			build,
			receiver,
			serial:   details::Serial {
				mainboard,
				controller,
			},
		})
	}

	#[doc(hidden)]
	#[cfg(target_os = "linux")]
	pub fn receive(&mut self, timeout: Duration) -> Res<(u8, &[u8])> {
		if self.handle.read_interrupt(self.address, &mut self.packet, timeout)? != 64 {
			return Err(Error::InvalidParameter);
		}

		Ok((self.packet[2], &self.packet[4 .. (self.packet[3] + 4) as usize]))
	}

	#[doc(hidden)]
	#[cfg(not(target_os = "linux"))]
	pub fn receive(&mut self, timeout: Duration) -> Res<(u8, &[u8])> {
		if self.handle.data().read(&mut self.packet[1..], timeout)?.unwrap_or(0) != 64 {
			return Err(Error::InvalidParameter);
		}

		Ok((self.packet[3], &self.packet[5 .. (self.packet[4] + 5) as usize]))
	}

	/// Get the current state of the controller.
	pub fn state(&mut self, timeout: Duration) -> Res<(State, Vec<u8>)> {
		let (id, buffer) = self.receive(timeout)?;
		let buf_export = buffer.to_vec();

		let state = State::parse(id, Cursor::new(buffer))?;

		if let State::Power(true) = state {
			self.reset()?;
		}

		Ok((state, buf_export))
	}
}
