extern "C" {
    static _xdp_program_start: u8;
    static _xdp_program_end: u8;
}

use rxdp;

use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn get_embedded_program() -> &'static [u8] {
    unsafe {
        let start_address = (&_xdp_program_start as *const u8) as usize;
        let end_address = (&_xdp_program_end as *const u8) as usize;
        let embedded_program_size = end_address - start_address;
        let header_ptr = (&_xdp_program_start as *const u8);
        let result = std::slice::from_raw_parts(header_ptr, embedded_program_size);

        println!("Embedded program size: {}", embedded_program_size);
        result
    }
}

fn main() -> Result<(), Error> {
    let embedded_blocker = get_embedded_program();

    let obj_path = "bpf/block.o";
    let obj = match rxdp::XDPObject::new(obj_path) {
        Ok(obj) => obj,
        Err(err) => panic!("{:?}", err),
    };

    let loaded_obj = match obj.load() {
        Ok(obj) => obj,
        Err(err) => panic!("{:?}", err),
    };

    let interface_name = "eth0";
    let loaded_program_name = "selective_drop";

    let loaded_prog = match loaded_obj.get_program(loaded_program_name) {
        Ok(program) => program,
        Err(err) => {
            // TODO: Unload the object.
            panic!("{:?}", err)
        }
    };

    if let Err(err) = loaded_prog.attach_to_interface(interface_name, rxdp::AttachFlags::SKB_MODE) {
        // TODO: Unload the object.
        panic!("{:?}", err)
    }

    println!("Beginning to block.");
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
    while !term.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    println!("Done blocking.");

    if let Err(err) = loaded_prog.detach_from_interface(interface_name) {
        panic!("{:?}", err);
    };

    Ok(())
}
