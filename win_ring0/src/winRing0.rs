use win_kernel_driver::WinKernelDriver;
use win_kernel_driver::DriverBuilder;
use super::ioctl::IOCTL;
use winapi::shared::minwindef::{DWORD};

/// WinRing0 driver
pub struct WinRing0 { 
    driver: WinKernelDriver
}

impl<'a> WinRing0 {
    pub fn new() -> Self {
        let driver_x64 = include_bytes!("../winRing0x64.sys");
        let driver_x86 = include_bytes!("../winRing0.sys");

        let driver = DriverBuilder::new()
            .set_device_description("Rust winRing0 driver")
            .set_device_id("WinRing0_1_2_0")
            .set_device_type(40000)
            .set_driver_bin(driver_x64.to_vec())
            .build().unwrap();

        WinRing0 {
            driver: driver
        }
    }

    /// Install the winRing0 driver.
    pub fn install(&self) -> Result<(), String> {
        return self.driver.install();
    }

    /// Open the winRing0 driver for communication
    pub fn open(&mut self) -> Result<(), String> {
        return self.driver.open();
    }

    /// Close the winRing0 driver handle
    pub fn close(&mut self) -> Result<(), String> {
        self.driver.close()
    }

    /// Uninstall the winRing0 driver
    pub fn uninstall(&mut self) -> Result<(), String> {
        self.driver.uninstall()
    }

    /// Read an MSR register
    pub fn readMsr(&self, msr: DWORD) -> Result<u64, String> {
        match self.driver.io(IOCTL::OLS_READ_MSR as u32, msr) {
            Ok(res) => { return Ok(res); }
            Err(err) => { return Err(format!("Error reading msr: {}", err)); }
        }
    }

    /// Raw IO function. See [WinKernelDriver::io] for more information
    pub fn io(&self, ioctl: IOCTL, in_buffer: u32) -> Result<u64, String> {
        match self.driver.io(ioctl as u32, in_buffer) {
            Ok(res) => { return Ok(res); },
            Err(err) => { return Err(format!("Error doing IO: {}", err)); }
        }
    }
}
