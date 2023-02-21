#![feature(horizon_thread_ext)]

use ctru::prelude::*;

use std::cell::RefCell;
use std::os::horizon::thread::BuilderExt;

std::thread_local! {
    static MY_LOCAL: RefCell<&'static str> = RefCell::new("initial value");
}

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::init().expect("Couldn't obtain GFX controller");
    gfx.top_screen.borrow_mut().set_wide_mode(true);
    let hid = Hid::init().expect("Couldn't obtain HID controller");
    let apt = Apt::init().expect("Couldn't obtain APT controller");
    let _console = Console::init(gfx.top_screen.borrow_mut());

    // Give ourselves up to 30% of the system core's time
    apt.set_app_cpu_time_limit(30)
        .expect("Failed to enable system core");

    MY_LOCAL.with(|local| {
        println!("Initial value on main thread: {}", local.borrow());
        *local.borrow_mut() = "second value";
    });
    MY_LOCAL.with(|local| {
        println!("Value on main thread after mutation: {}", local.borrow());
    });

    std::thread::Builder::new()
        .processor_id(1)
        .spawn(move || {
            MY_LOCAL.with(|local| {
                println!("Initial value on second thread: {}", local.borrow());
                *local.borrow_mut() = "third value";
            });
            MY_LOCAL.with(|local| {
                println!("Value on second thread after mutation: {}", local.borrow());
            });
        })
        .expect("Failed to create thread")
        .join()
        .expect("Failed to join on thread");

    MY_LOCAL.with(|local| {
        println!(
            "Value on main thread after second thread exits: {}",
            local.borrow()
        );
    });

    println!("Press Start to exit");

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }

        gfx.flush_buffers();
        gfx.swap_buffers();
        gfx.wait_for_vblank();
    }
}
