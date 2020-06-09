use openhardware::hardware::get_cpu;
use openhardware::hardware::cpu::UpdateTypes;
use openhardware::hardware::cpu::CPU;
use win_ring0::WinRing0;

fn main() {

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

    {
        let mut cpu = get_cpu(&mut r0).unwrap();
        cpu.update(UpdateTypes::All);
    }

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
