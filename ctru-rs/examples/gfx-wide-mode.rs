use ctru::prelude::*;

fn main() {
    ctru::use_panic_handler();

    let apt = Apt::init().unwrap();
    let hid = Hid::init().unwrap();
    let gfx = Gfx::init().unwrap();
    let mut console = Console::init(gfx.top_screen.borrow_mut());

    println!("Press A to enable/disable wide screen mode.");

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }

        if hid.keys_down().contains(KeyPad::KEY_A) {
            drop(console);

            let wide_mode = gfx.top_screen.borrow().get_wide_mode();
            gfx.top_screen.borrow_mut().set_wide_mode(!wide_mode);

            console = Console::init(gfx.top_screen.borrow_mut());
            println!("Press A to enable/disable wide screen mode.");
        }

        gfx.flush_buffers();
        gfx.swap_buffers();
        gfx.wait_for_vblank();
    }
}
