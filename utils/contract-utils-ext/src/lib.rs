#![no_std]
#![feature(once_cell)]

extern crate alloc;

mod commission_control;
mod minters_control;

pub use commission_control::{Commission, Commissions};
pub use minters_control::MinterControl;
