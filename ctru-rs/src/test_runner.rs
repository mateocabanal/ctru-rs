//! Custom test runner for building/running unit tests on the 3DS.

extern crate test;

use std::io;

use test::{ColorConfig, OutputFormat, TestDescAndFn, TestFn, TestOpts};

use crate::console::Console;
use crate::gfx::Gfx;
use crate::services::hid::{Hid, KeyPad};
use crate::services::Apt;

/// A custom runner to be used with `#[test_runner]`. This simple implementation
/// runs all tests in series, "failing" on the first one to panic (really, the
/// panic is just treated the same as any normal application panic).
pub(crate) fn run(tests: &[&TestDescAndFn]) {
    let gfx = Gfx::init().unwrap();
    let hid = Hid::init().unwrap();
    let apt = Apt::init().unwrap();

    let mut top_screen = gfx.top_screen.borrow_mut();
    top_screen.set_wide_mode(true);
    let _console = Console::init(top_screen);

    let opts = TestOpts {
        force_run_in_process: true,
        run_tests: true,
        // TODO: color doesn't work because of TERM/TERMINFO.
        // With RomFS we might be able to fake this out nicely...
        color: ColorConfig::AutoColor,
        format: OutputFormat::Pretty,
        // Hopefully this interface is more stable vs specifying individual options,
        // and parsing the empty list of args should always work, I think.
        // TODO Ideally we could pass actual std::env::args() here too
        ..test::test::parse_opts(&[]).unwrap().unwrap()
    };
    // Use the default test implementation with our hardcoded options
    let _success = run_static_tests(&opts, tests).unwrap();

    // Make sure the user can actually see the results before we exit
    println!("Press START to exit.");

    while apt.main_loop() {
        gfx.flush_buffers();
        gfx.swap_buffers();
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::KEY_START) {
            break;
        }
    }
}

/// Adapted from [`test::test_main_static`] and [`test::make_owned_test`].
fn run_static_tests(opts: &TestOpts, tests: &[&TestDescAndFn]) -> io::Result<bool> {
    let tests = tests.iter().map(make_owned_test).collect();
    test::run_tests_console(opts, tests)
}

/// Clones static values for putting into a dynamic vector, which test_main()
/// needs to hand out ownership of tests to parallel test runners.
///
/// This will panic when fed any dynamic tests, because they cannot be cloned.
fn make_owned_test(test: &&TestDescAndFn) -> TestDescAndFn {
    match test.testfn {
        TestFn::StaticTestFn(f) => TestDescAndFn {
            testfn: TestFn::StaticTestFn(f),
            desc: test.desc.clone(),
        },
        TestFn::StaticBenchFn(f) => TestDescAndFn {
            testfn: TestFn::StaticBenchFn(f),
            desc: test.desc.clone(),
        },
        _ => panic!("non-static tests passed to test::test_main_static"),
    }
}

/// The following functions are stubs needed to link the test library,
/// but do nothing because we don't actually need them for the runner to work.
mod link_fix {
    #[no_mangle]
    extern "C" fn execvp(
        _argc: *const libc::c_char,
        _argv: *mut *const libc::c_char,
    ) -> libc::c_int {
        -1
    }

    #[no_mangle]
    extern "C" fn pipe(_fildes: *mut libc::c_int) -> libc::c_int {
        -1
    }

    #[no_mangle]
    extern "C" fn sigemptyset(_arg1: *mut libc::sigset_t) -> ::libc::c_int {
        -1
    }
}
