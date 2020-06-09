//! `win-kernel-driver` is a crate for installing and communicating with
//! windows device drivers.
//! 
//! # Information
//! 
//! This crate is based off of the [KernelDriver class](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/Hardware/KernelDriver.cs)
//! from OpenHardwareMonitor.
//! 
//! For example usage see [WinKernelDriver], [DriverBuilder], and [io_control_code]
//!
mod utils;
mod driver;

pub use driver::WinKernelDriver;
pub use driver::DriverBuilder;
pub use driver::Access;
pub use driver::Method;
pub use driver::io_control_code;
