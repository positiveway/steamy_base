//! Steam controller handling library.

const VENDOR_ID: u16 = 0x28de;
const PRODUCT_ID: [u16; 2] = [0x1102, 0x1142];
const ENDPOINT: [u8; 2] = [3, 2];
const INDEX: [u16; 2] = [2, 1];

mod manager;

pub use manager::Manager;

mod controller;

pub use controller::Controller;

mod feedback;

pub use feedback::Feedback;

mod sensors;

pub use sensors::Sensors;

mod led;

pub use led::Led;

pub mod sound;

pub use sound::Sound;

mod calibrate;

pub use calibrate::Calibrate;

mod lizard;

pub use lizard::Lizard;

pub mod button;

pub use button::Button;

mod state;

pub use state::{State, Axis, Trigger, Pad, Angles};

pub mod details;

pub use details::Details;
