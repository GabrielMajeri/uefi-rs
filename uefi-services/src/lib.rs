//! This crate simplifies the writing of higher-level code for UEFI.
//!
//! It initializes the memory allocation and logging crates,
//! allowing code to use Rust's data structures and to log errors.
//!
//! It also stores a global reference to the UEFI system table,
//! in order to reduce the redundant passing of references to it.
//!
//! Library code can simply use global UEFI functions
//! through the reference provided by `system_table`.

#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(panic_info_message)]

// These crates are required.
extern crate rlibc;

// Core types.
extern crate uefi;

// Logging support
extern crate uefi_logger;

// Allocator support.
extern crate uefi_alloc;

#[macro_use]
extern crate log;

use uefi::table::SystemTable;

/// Reference to the system table.
static mut SYSTEM_TABLE: Option<&'static SystemTable> = None;

/// Obtains a reference to the system table.
///
/// This is meant to be used by higher-level libraries,
/// which want a convenient way to access the system table singleton.
///
/// `init` must have been called first by the UEFI app.
pub fn system_table() -> &'static SystemTable {
    unsafe { SYSTEM_TABLE.expect("The uefi-services library has not yet been initialized") }
}

/// Initialize the UEFI utility library.
///
/// This must be called as early as possible,
/// before trying to use logging or memory allocation capabilities.
pub fn init(st: &'static SystemTable) {
    unsafe {
        // Avoid double initialization.
        if SYSTEM_TABLE.is_some() {
            return;
        }

        SYSTEM_TABLE = Some(st);
    }

    init_logger();
    init_alloc();
}

fn init_logger() {
    let st = system_table();

    static mut LOGGER: Option<uefi_logger::Logger> = None;

    let stdout = st.stdout();

    // Construct the logger.
    let logger = unsafe {
        LOGGER = Some(uefi_logger::Logger::new(stdout));

        LOGGER.as_ref().unwrap()
    };

    // Set the logger.
    log::set_logger(logger).unwrap(); // Can only fail if already initialized.

    // Log everything.
    log::set_max_level(log::LevelFilter::Info);
}

fn init_alloc() {
    let st = system_table();

    uefi_alloc::init(st.boot);
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
        if let Some(message) = info.message() {
            error!("{}", message);
        }
    }

    // Give the user some time to read the message
    if let Some(st) = unsafe { SYSTEM_TABLE } {
        // FIXME: Check if boot-time services have been exited too
        st.boot.stall(10_000_000);
    } else {
        let mut dummy = 0u64;
        // FIXME: May need different counter values in debug & release builds
        for i in 0..300_000_000 {
            unsafe {
                core::ptr::write_volatile(&mut dummy, i);
            }
        }
    }

    // If running in QEMU, use the f4 exit port to signal the error and exit
    if cfg!(feature = "qemu") {
        use x86_64::instructions::port::Port;
        let mut port = Port::<u32>::new(0xf4);
        unsafe {
            port.write(42);
        }
    }

    // If the system table is available, use UEFI's standard shutdown mechanism
    if let Some(st) = unsafe { SYSTEM_TABLE } {
        use uefi::table::runtime::ResetType;
        st.runtime
            .reset(ResetType::Shutdown, uefi::Status::Aborted, None)
    }

    // If we don't have any shutdown mechanism handy, the best we can do is loop
    error!("Could not shut down, please power off the system manually...");

    loop {
        unsafe {
            // Try to at least keep CPU from running at 100%
            asm!("hlt" :::: "volatile");
        }
    }
}

#[alloc_error_handler]
fn out_of_memory(layout: ::core::alloc::Layout) -> ! {
    panic!(
        "Ran out of free memory while trying to allocate {:#?}",
        layout
    );
}
