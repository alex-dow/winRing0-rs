use std::path::PathBuf;
use std::ffi::OsString;
use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::{c_void, CString};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::Write;

use windows_service::{
    service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType, Service, ServiceStatus, ServiceState},
    service_manager::{ServiceManager, ServiceManagerAccess}
};

use winapi::um::winnt;
use winapi::um::fileapi;
use winapi::um::handleapi;
use winapi::shared::minwindef::{DWORD};
use winapi::um::ioapiset;
use winapi::um::errhandlingapi;
use winapi::um::winioctl;


/// IO Method
#[repr(u32)]
pub enum Method {
    BUFFERED = 0,
    INDIRECT = 1,
    OUTDIRECT = 2,
    NEITHER = 3
}

/// IO Access
#[repr(u32)]
pub enum Access {
    ANY = 0,
    READ = 1,
    WRITE = 2
}

/// Creates an IOCTL code
/// 
/// # Arguments
/// 
/// * `device_type` - The device type is a 32bit integer. It must match the kernel driver
///                   you're loading.
/// * `function`    - The function code you want to send to the driver. Consult your driver's
///                   documentation for the available codes.DriverBuilder
/// * `method`      - The IO method to use
/// * `access`      - The access level (read/write/any) to use
/// 
/// # Example
/// ```
/// let device = 0x00000022; // FILE_DEVICE_UNKNOWN
/// let function = 0x800; // Some function code defined by the driver
/// 
/// // Generate an IO control code for a buffered read to the driver.
/// let ioctl = io_control_code(device, function, Method.BUFFERED, Access.READ);
/// ```
pub const fn io_control_code(device_type: u32, function: u32, method: Method, access: Access) -> u32 {
    (device_type << 16) | ((access as u32) << 14) | (function << 2) | (method as u32)
}

/// Use this to build a kernel driver object you can interact with
/// 
/// # Example
/// ```
/// let driver_bin = include_bytes!("../winRing0.sys");
/// let driver = DriverBuilder::new()
///              .set_device_description("winRing0 driver")
///              .set_device_id("WinRing0_1_2_0")
///              .set_driver_bin(driver_bin.to_vec())
///              .build().unwrap();
/// ```
pub struct DriverBuilder {
    device_id: &'static str,
    device_description: &'static str,
    device_type: DWORD,
    driver_path: PathBuf,
    driver_bin: Vec<u8>
}

impl DriverBuilder {
    pub fn new() -> Self {
        DriverBuilder {
            device_id: "",
            device_description: "",
            device_type: winioctl::FILE_DEVICE_UNKNOWN,
            driver_path: PathBuf::new(),
            driver_bin: vec![]
        }
    }

    /// Set the device id (required)
    pub fn set_device_id(mut self, device_id: &'static str) -> Self {
        self.device_id = device_id;
        return self;
    }

    /// Set the device description (required)
    pub fn set_device_description(mut self, device_description: &'static str) -> Self {
        self.device_description = device_description;
        return self;
    }

    /// Set the device type (defaults to FILE_DEVICE_UNKNOWN (0x00000022))
    pub fn set_device_type(mut self, device_type: DWORD) -> Self {
        self.device_type = device_type;
        return self;
    }

    /// Set the path to the driver file (optional)
    pub fn set_driver_path(mut self, driver_path: PathBuf) -> Self {
        self.driver_path = driver_path;
        return self;
    }

    /// Use a bytearray for the driver. It will be written to a temporary location
    /// Useful with the !include_bin macro
    pub fn set_driver_bin(mut self, driver_bin: Vec<u8>) -> Self {
        self.driver_bin = driver_bin;
        return self;
    }

    /// Build a WinKernelDriver instance
    pub fn build(&mut self) -> Result<WinKernelDriver, String> {

        if self.device_id.len() == 0 {
            return Err("Device ID needs to be set!".to_owned());
        }

        if self.driver_bin.len() == 0 && self.driver_path.components().count() == 0 {
            return Err("Either a path to the driver file, or a binary array of the driver file, must be set".to_owned());
        }

        if self.driver_bin.len() > 0 {
            let mut dir = PathBuf::from(env::temp_dir());
            dir.push(format!("{}.sys", self.device_id));

            let mut f = File::create(&dir).unwrap();
            
            let driver_bin_buffer: &[u8] = &&self.driver_bin;
            f.write_all(driver_bin_buffer).unwrap();

            self.driver_path = dir;
        }

        let driver = WinKernelDriver {
            service_description: self.device_description,
            driver_path: PathBuf::from(self.driver_path.clone()),
            device_id: self.device_id,
            device: None
        };

        Ok(driver)
    }
}

/// A handle to a windows kernel driver.
/// 
/// The driver is installed as a service. Once installed io handles
/// can be opened / closed with the driver.
/// 
/// # Example
/// 
/// ```
/// let driver_bin = include_bytes!("../winRing0.sys");
/// let driver = DriverBuilder::new()
///              .set_device_description("winRing0 driver")
///              .set_device_id("WinRing0_1_2_0")
///              .set_device_type(40000)
///              .set_driver_bin(driver_bin.to_vec())
///              .build().unwrap();
/// 
/// driver.install().unwrap();
/// driver.open().unwrap();
/// 
/// // Read MSR_TEMPERATURE_TARGET on intel CPUs
/// let ioctl = io_control_code(40000, 0x800, Method::BUFFERED, Access::ANY);
/// let out = device.io(ioctl, 0x1a2).unwrap();
/// let edx = ((out >> 32) & 0xFFFFFFFF) as u32;
/// let eax = (out & 0xFFFFFFFF) as u32;
/// 
/// let temp_target = (eax >> 16) & 0xff;
/// 
/// println!("MSR_TEMPERATURE_TARGET: {}", temp_target);
/// 
/// driver.close().unwrap();
/// driver.uninstall().unwrap();
/// ```
pub struct WinKernelDriver {
    service_description: &'static str,
    driver_path: PathBuf,
    device_id: &'static str,
    device: Option<winnt::HANDLE>
}

impl WinKernelDriver {

    /// Install the driver service
    pub fn install(&self) -> Result<(), String> {

        let manager_access = ServiceManagerAccess::all();
        let service_manager_res = ServiceManager::local_computer(None::<&str>, manager_access);
        let service_manager: ServiceManager;

        match service_manager_res {
            Ok(svcman) => { service_manager = svcman; },
            Err(err) => { return Err(format!("Unable to connec to service manager: {}", err)); }
        }

        let service_info = ServiceInfo {
            name: OsString::from(self.device_id),
            display_name: OsString::from(self.service_description),
            service_type: ServiceType::KERNEL_DRIVER,
            start_type: ServiceStartType::OnDemand,
            error_control: ServiceErrorControl::Normal,
            executable_path: self.driver_path.clone(),
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None,
            account_password: None
        };

        let service = service_manager.create_service(service_info, ServiceAccess::all());
        match service {
            Ok(svc) => {
                let r = svc.start(&[OsString::from("")]);
                match r {
                    Ok(_) => { },
                    Err(err) => { return Err(format!("Failed to start service: {:?}", err)); }
                };
            },
            Err(err) => { return Err(format!("Service error: {:?}", err)); }
        };

        Ok(())
    }
    
    /// Uninstall the driver service
    pub fn uninstall(&self) -> Result<(), String> {
        let manager_access = ServiceManagerAccess::all();
        let service_manager_res = ServiceManager::local_computer(None::<&str>, manager_access);
        let service_manager: ServiceManager;

        match service_manager_res {
            Ok(manager) => { service_manager = manager; },
            Err(err) => { return Err(format!("Error getting service manager: {:?}", err)); }
        }

        let service: Service;

        let open_res = service_manager.open_service(self.device_id, ServiceAccess::all());
        match open_res {
            Ok(svc) => { service = svc; },
            Err(err) => { return Err(format!("Error opening service: {:?}", err)); }
        }

        let service_status: ServiceStatus;
        match service.query_status() {
            Ok(status) => service_status = status,
            Err(err) => { return Err(format!("Error querying service status: {:?}", err)); }
        }

        if service_status.current_state != ServiceState::Stopped {
            match service.stop() {
                Ok(_) => { },
                Err(err) => { return Err(format!("Error stopping service: {:?}", err)); }
            }
        }

        match service.delete() {
            Ok(()) => { return Ok(()); },
            Err(err) => { return Err(format!("Error deleting service: {:?}", err)); }
        }
    }
    
    /// Open the driver service. Once opened the [WinKernelDriver::io()] function can be called.
    pub fn open(&mut self) -> Result<(), String> {

        if self.opened() {
            return Err("Driver already opened".to_string());
        }

        let mut driver_path_t: String = r"\\.\".to_string();
        driver_path_t.push_str(self.device_id);
        let driver_path = driver_path_t.as_str();

        unsafe {
            let device: winnt::HANDLE = fileapi::CreateFileA(
                CString::new(driver_path).unwrap().as_ptr(),
                winnt::GENERIC_READ | winnt::GENERIC_WRITE,
                0,
                null_mut(),
                fileapi::OPEN_EXISTING,
                winnt::FILE_ATTRIBUTE_NORMAL,
                null_mut()
            );

            if device == handleapi::INVALID_HANDLE_VALUE {
                return Err("Error occurred getting handle on kernel driver".to_string());
            } else {
                println!("Handle created");
            }
            
            self.device = Some(device);
        }

        Ok(())
    }    

    /// Check to see if there is an open handle to the driver
    pub fn opened(&self) -> bool {
        match self.device {
            Some(_) => { return true; }
            None => { return false; }
        }
    }
    
    /// Close the open handle to the driver
    pub fn close(&mut self) -> Result<(), String> {

        if !self.opened() {
            return Err("Driver not opened".to_string());
        }

        let handle = self.device.unwrap();
        unsafe {
            handleapi::CloseHandle(handle);
        }        

        Ok(())
    }

    /// Perform an IO command on the driver.
    /// 
    /// To know which IO commands are available, you must check with the driver
    /// you are trying to work with. IO control codes should be made with
    /// the [io_control_code] function.
    pub fn io(&self, ioctl_code: u32, mut in_buffer: u32) -> Result<u64, String> {
        if !self.opened() {
            return Err("Driver not opened!".to_string());
        }

        let device = self.device.unwrap() as winnt::HANDLE;

        let mut out_buffer = [0u8; size_of::<u64>()];
        let in_buffer_bytes = in_buffer.to_be_bytes();

        let in_buffer_size = u32::try_from(in_buffer_bytes.len()).unwrap() as DWORD;
        let out_buffer_size = out_buffer.len() as u32;
        let mut out_buffer_written: DWORD = 0;

        unsafe {
            let res = ioapiset::DeviceIoControl(
                device,
                ioctl_code,
                &mut in_buffer as *mut _ as *mut c_void,
                in_buffer_size,
                out_buffer.as_mut_ptr() as *mut _,
                out_buffer_size,
                &mut out_buffer_written,
                null_mut()
            );

            if res != 0 {
                let o = u64::from_le_bytes(out_buffer);
                return Ok(o);
            } else {
                let last_error = errhandlingapi::GetLastError();
                return Err(format!("DeviceIoControl - Unable to write command {:x}. Last error code: {:x}", ioctl_code, last_error));
            }
        }
    }    
}
