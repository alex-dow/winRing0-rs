use win_ring0::WinRing0;

pub fn main() {
    let mut r0: Box<WinRing0> = Box::from(WinRing0::new());

    println!("Installing ring0 driver");
    match r0.install() {
        Ok(()) => { println!("Driver installed"); }
        Err(err) => { println!("Error: {}", err); }
    }

    println!("Opening ring0 driver");
    match r0.open() {
        Ok(()) => { println!("Driver opened"); }
        Err(err) => { println!("Error: {}", err); }
    }

    println!("Trying to get tjMax value, should work on most Intel CPUs");
    // MSR_TEMPERATURE_TARGET
    let msr = 0x1a2;
    let out = r0.readMsr(msr).unwrap();

    let _edx = ((out >> 32) & 0xffffffff) as u32;
    let eax = (out & 0xffffffff) as u32;
    let tj_max = (eax >> 16) & 0xff;

    println!("MSR Value: {}", tj_max);

    println!("Closing ring0 driver");
    match r0.close() {
        Ok(()) => { println!("Driver closed"); }
        Err(err) => { println!("Error: {}", err); }
    }

    println!("Uninstall ring0 driver");
    match r0.uninstall() {
        Ok(()) => { println!("Driver uninstalled"); }
        Err(err) => { println!("Error: {}", err); }
    }    
}
