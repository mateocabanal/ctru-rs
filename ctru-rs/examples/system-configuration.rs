use ctru::prelude::*;
use ctru::services::cfgu::Cfgu;

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::init().expect("Couldn't obtain GFX controller");
    let hid = Hid::init().expect("Couldn't obtain HID controller");
    let apt = Apt::init().expect("Couldn't obtain APT controller");
    let cfgu = Cfgu::init().expect("Couldn't obtain CFGU controller");
    let _console = Console::init(gfx.top_screen.borrow_mut());

    println!("\x1b[0;0HRegion: {:?}", cfgu.get_region().unwrap());
    println!("\x1b[10;0HLanguage: {:?}", cfgu.get_language().unwrap());
    println!("\x1b[20;0HModel: {:?}", cfgu.get_model().unwrap());

    // Main loop
    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }
        // Flush and swap framebuffers
        gfx.flush_buffers();
        gfx.swap_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}
