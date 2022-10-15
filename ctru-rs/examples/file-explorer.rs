//! A file explorer which shows off using standard library file system APIs to
//! read the SD card.

use ctru::applets::swkbd::{Button, Swkbd};
use ctru::prelude::*;

use std::fs::DirEntry;
use std::os::horizon::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn main() {
    ctru::init();
    let apt = Apt::init().unwrap();
    let hid = Hid::init().unwrap();
    let gfx = Gfx::init().unwrap();

    #[cfg(all(feature = "romfs", romfs_exists))]
    let _romfs = ctru::romfs::RomFS::init().unwrap();

    FileExplorer::init(&apt, &hid, &gfx).run();
}

struct FileExplorer<'a> {
    apt: &'a Apt,
    hid: &'a Hid,
    gfx: &'a Gfx,
    console: Console<'a>,
    path: PathBuf,
    entries: Vec<DirEntry>,
    running: bool,
}

impl<'a> FileExplorer<'a> {
    fn init(apt: &'a Apt, hid: &'a Hid, gfx: &'a Gfx) -> Self {
        let mut top_screen = gfx.top_screen.borrow_mut();
        top_screen.set_wide_mode(true);
        let console = Console::init(top_screen);

        FileExplorer {
            apt,
            hid,
            gfx,
            console,
            path: PathBuf::from("/"),
            entries: Vec::new(),
            running: false,
        }
    }

    fn run(&mut self) {
        self.running = true;
        self.print_menu();

        while self.running && self.apt.main_loop() {
            self.hid.scan_input();
            let input = self.hid.keys_down();

            if input.contains(KeyPad::KEY_START) {
                break;
            } else if input.contains(KeyPad::KEY_B) && self.path.components().count() > 1 {
                self.path.pop();
                self.console.clear();
                self.print_menu();
            } else if input.contains(KeyPad::KEY_A) {
                self.get_input_and_run(Self::set_next_path);
            } else if input.contains(KeyPad::KEY_X) {
                self.get_input_and_run(Self::set_exact_path);
            }

            self.gfx.flush_buffers();
            self.gfx.swap_buffers();
            self.gfx.wait_for_vblank();
        }
    }

    fn print_menu(&mut self) {
        match std::fs::metadata(&self.path) {
            Ok(metadata) => {
                println!(
                    "Viewing {} (size {} bytes, mode {:#o})",
                    self.path.display(),
                    metadata.len(),
                    metadata.st_mode(),
                );

                if metadata.is_file() {
                    self.print_file_contents();
                    // let the user continue navigating from the parent dir
                    // after dumping the file
                    self.path.pop();
                    self.print_menu();
                    return;
                } else if metadata.is_dir() {
                    self.print_dir_entries();
                } else {
                    println!("unsupported file type: {:?}", metadata.file_type());
                }
            }
            Err(e) => {
                println!("Failed to read {}: {}", self.path.display(), e)
            }
        };

        println!("Start to exit, A to select an entry by number, B to go up a directory, X to set the path.");
    }

    fn print_dir_entries(&mut self) {
        let dir_listing = std::fs::read_dir(&self.path).expect("Failed to open path");
        self.entries = Vec::new();

        for (i, entry) in dir_listing.enumerate() {
            match entry {
                Ok(entry) => {
                    println!("{:2} - {}", i, entry.file_name().to_string_lossy());
                    self.entries.push(entry);

                    if (i + 1) % 20 == 0 {
                        self.wait_for_page_down();
                    }
                }
                Err(e) => {
                    println!("{} - Error: {}", i, e);
                }
            }
        }
    }

    fn print_file_contents(&mut self) {
        match std::fs::read_to_string(&self.path) {
            Ok(contents) => {
                println!("File contents:\n{0:->80}", "");
                println!("{contents}");
                println!("{0:->80}", "");
            }
            Err(err) => {
                println!("Error reading file: {}", err);
            }
        }
    }

    /// Paginate output
    fn wait_for_page_down(&mut self) {
        println!("Press A to go to next page, or Start to exit");

        while self.apt.main_loop() {
            self.hid.scan_input();
            let input = self.hid.keys_down();

            if input.contains(KeyPad::KEY_A) {
                break;
            }

            if input.contains(KeyPad::KEY_START) {
                self.running = false;
                return;
            }

            self.gfx.wait_for_vblank();
        }
    }

    fn get_input_and_run(&mut self, action: impl FnOnce(&mut Self, String)) {
        let mut keyboard = Swkbd::default();
        let mut new_path_str = String::new();

        match keyboard.get_utf8(&mut new_path_str) {
            Ok(Button::Right) => {
                // Clicked "OK"
                action(self, new_path_str);
            }
            Ok(Button::Left) => {
                // Clicked "Cancel"
            }
            Ok(Button::Middle) => {
                // This button wasn't shown
                unreachable!()
            }
            Err(e) => {
                panic!("Error: {:?}", e)
            }
        }
    }

    fn set_next_path(&mut self, next_path_index: String) {
        let next_path_index: usize = match next_path_index.parse() {
            Ok(index) => index,
            Err(e) => {
                println!("Number parsing error: {}", e);
                return;
            }
        };

        let next_entry = match self.entries.get(next_path_index) {
            Some(entry) => entry,
            None => {
                println!("Input number of bounds");
                return;
            }
        };

        self.console.clear();
        self.path = next_entry.path();
        self.print_menu();
    }

    fn set_exact_path(&mut self, new_path_str: String) {
        let new_path = Path::new(&new_path_str);
        if !new_path.is_dir() {
            println!("Not a directory: {}", new_path_str);
            return;
        }

        self.console.clear();
        self.path = new_path.to_path_buf();
        self.print_menu();
    }
}
