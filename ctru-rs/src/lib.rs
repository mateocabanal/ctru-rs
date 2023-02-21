#![crate_type = "rlib"]
#![crate_name = "ctru"]
#![feature(test)]
#![feature(custom_test_frameworks)]
#![feature(try_trait_v2)]
#![feature(allocator_api)]
#![feature(nonnull_slice_from_raw_parts)]
#![test_runner(test_runner::run)]

// Nothing is imported from these crates but their inclusion here assures correct linking of the missing implementations.
extern crate linker_fix_3ds;
extern crate pthread_3ds;

#[no_mangle]
#[cfg(feature = "big-stack")]
static __stacksize__: usize = 2 * 1024 * 1024; // 2MB

/// Activate ´ctru-rs´' default panic handler.
///
/// With this implementation, the main thread will stop and try to print debug info to an available [console::Console].
/// In case it fails to find an active [console::Console], the program will just exit.
///
/// # Notes
///
/// When ´test´ is enabled, this function won't do anything, as it should be overridden by the ´test´ environment.
pub fn use_panic_handler() {
    #[cfg(not(test))]
    panic_hook_setup();
}

#[cfg(not(test))]
fn panic_hook_setup() {
    use crate::services::hid::{Hid, KeyPad};
    use std::panic::PanicInfo;

    let main_thread = std::thread::current().id();

    // Panic Hook setup
    let default_hook = std::panic::take_hook();
    let new_hook = Box::new(move |info: &PanicInfo| {
        default_hook(info);

        // Only for panics in the main thread
        if main_thread == std::thread::current().id() && console::Console::exists() {
            println!("\nPress SELECT to exit the software");

            match Hid::init() {
                Ok(hid) => loop {
                    hid.scan_input();
                    let keys = hid.keys_down();
                    if keys.contains(KeyPad::KEY_SELECT) {
                        break;
                    }
                },
                Err(e) => println!("Error while intializing Hid controller during panic: {e}"),
            }
        }
    });
    std::panic::set_hook(new_hook);
}

pub mod applets;
pub mod console;
pub mod error;
pub mod gfx;
pub mod linear;
pub mod mii;
pub mod prelude;
pub mod services;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "romfs", romfs_exists))] {
        pub mod romfs;
    } else {
        pub mod romfs {
            //! The RomFS folder has not been detected and/or the `romfs` feature has not been enabled.
            //!
            //! Configure the path in Cargo.toml (the default path is "romfs"). Paths are relative to the
            //! `CARGO_MANIFEST_DIR` environment variable, which is the directory containing the manifest of
            //! your package.
            //!
            //! ```toml
            //! [package.metadata.cargo-3ds]
            //! romfs_dir = "romfs"
            //! ```
        }
    }
}

#[cfg(test)]
mod test_runner;

pub use crate::error::{Error, Result};
