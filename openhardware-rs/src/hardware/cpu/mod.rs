use winRing0::WinRing0;
extern crate raw_cpuid;
use raw_cpuid::CpuId;
use std::rc::Rc;

pub mod intel;
use intel::IntelCPU;

pub enum CpuUpdateTypes {
    Frequency,
    Temperature,
    Load,
    All
}
pub struct CPUDevice { 
    driver: Option<Rc<WinRing0>>,
    cpu: Option<Rc<dyn CPU + 'static>>
}

impl CPUDevice {
    pub fn new(self) -> Self {
        CPUDevice {
            driver: None,
            cpu: None
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        let cpuid = raw_cpuid::CpuId::new();
        let vendor_info = cpuid.get_vendor_info().unwrap();
        let ex_vendor_info = cpuid.get_feature_info().unwrap();
        let family_id = ex_vendor_info.family_id();
        println!("vendor info: {} - family id: {}", vendor_info.as_string(), family_id);
        
        match vendor_info.as_string() {
            "GenuineIntel" => {
                let cpu = IntelCPU::new();

                if self.driver.is_some() {
                    let drv = self.driver.as_ref();
                    cpu.set_driver(drv.unwrap());
                }

                self.cpu = Some(Rc::from(cpu));

                return Ok(());
            },
            _ => {
                return Err(format!("CPU Arch {} not supported for now", vendor_info.as_string()));
            }
        }
    }
}

pub trait CPU {
    fn update(&mut self, updateType: CpuUpdateTypes);
    fn cores(&mut self) -> u8;
    fn set_driver(&mut self, driver: &Rc<WinRing0>);
}




