//! Utility functions for the most common UEFI patterns.
//!
//! This crate simply exports some extension traits
//! which add utility functions to various UEFI objects.

#![no_std]
#![feature(alloc, alloc_layout_extra)]

extern crate alloc;

mod boot;
mod file;

pub use self::boot::BootServicesExt;
pub use self::file::FileExt;
