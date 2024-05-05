use crate::{Controller};
use crate::{VENDOR_ID, PRODUCT_ID, ENDPOINT, INDEX};
use color_eyre::{Result};
use color_eyre::eyre::bail;

/// Controller manager.
pub struct Manager {
    usb: rusb::Context,
}

impl Manager {
    /// Create a new controller manager.
    pub fn new() -> Result<Manager> {
        Ok(Manager {
            usb: rusb::Context::new()?,
        })
    }

    /// Open a controller.
    pub fn open(&mut self) -> Result<Controller> {
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

                return Ok(Controller::new(device, handle, product, endpoint, index)?);
            }
        }

        bail!(rusb::Error::NoDevice);
    }
}
