use super::CPU;
use super::CpuUpdateTypes;
use winRing0::WinRing0;
use x86::msr::IA32_PACKAGE_THERM_STATUS;
use x86::msr::MSR_TEMPERATURE_TARGET;
use raw_cpuid::CpuId;
use std::rc::Rc;

pub struct IntelCPU {
    tj_max: u32,
    driver: Option<&'static Rc<WinRing0>>,
    cores: u8
}

impl<'a> IntelCPU {
    pub fn new() -> Self {
        IntelCPU {
            driver: None,
            tj_max: 0,
            cores: 0
        }
    }

    fn update_tjmax(&mut self) -> u32 {
        let out: u64;

        let eax: u32;
        let edx: u32;
        let mut result: u32 = 0;

        if self.driver.is_some() {
            let drv = self.driver.as_ref();
            out = drv.unwrap().readMsr(MSR_TEMPERATURE_TARGET).unwrap();
        
            println!("Output of READ_MSR: {}", out);
            //eax = x.checked_shl(32).unwrap_or(0);
            edx = ((out >> 32) & 0xFFFFFFFF) as u32;
            eax = (out & 0xFFFFFFFF) as u32;
    
    
            println!("eax: {} - edx: {}", eax, edx);
        
            result = (eax >> 16) & 0xff;
        }

        return result;
    }
}

impl CPU for IntelCPU {
    fn update(&mut self, updateType: CpuUpdateTypes) {
        self.update_tjmax();
    }

    fn cores(&mut self) -> u8 {
        return 3;
    }

    fn set_driver(&mut self, driver: &'static Rc<WinRing0>) {
        self.driver = Some(driver);
    }
}
