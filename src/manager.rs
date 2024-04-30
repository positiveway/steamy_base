#[cfg(target_os = "linux")]
use usb;

#[cfg(not(target_os = "linux"))]
use hid;

use crate::{Result as Res, Controller};
use crate::{VENDOR_ID, PRODUCT_ID, ENDPOINT, INDEX};

/// Controller manager.
pub struct Manager {
	#[cfg(target_os = "linux")]
	usb: usb::Context,

	#[cfg(not(target_os = "linux"))]
	hid: hid::Manager,
}

impl Manager {
	/// Create a new controller manager.
	#[cfg(target_os = "linux")]
	pub fn new() -> Res<Manager> {
		Ok(Manager {
			usb: usb::Context::new()?,
		})
	}

	#[cfg(not(target_os = "linux"))]
	pub fn new() -> Res<Manager> {
		Ok(Manager {
			hid: hid::init()?,
		})
	}

	/// Open a controller.
	#[cfg(target_os = "linux")]
	pub fn open(&mut self) -> Res<Controller> {

		for mut device in rusb::devices()?.iter() {
			let descriptor = device.device_descriptor()?;

			if descriptor.vendor_id() != VENDOR_ID {
				continue;
			}

			for (&product, (&endpoint, &index)) in PRODUCT_ID.iter().zip(ENDPOINT.iter().zip(INDEX.iter())) {
				if descriptor.product_id() != product {
					continue;
				}
				
				let handle = device.open()?;

				return Controller::new(device, handle, product, endpoint, index);
			}
		}

		return Err(usb::Error::NoDevice.into());
	}

	#[cfg(not(target_os = "linux"))]
	pub fn open(&self) -> Res<Controller> {
		for &product in &PRODUCT_ID {
			for device in self.hid.find(Some(VENDOR_ID), Some(product)) {
				if let Ok(handle) = device.open() {
					return Controller::new(handle, product);
				}
			}
		}

		return Err(hid::Error::NotFound.into());
	}
}
