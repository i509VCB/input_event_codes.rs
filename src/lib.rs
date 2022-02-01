//! # Input event codes
//!
//! This library provides constants representing the input event codes generated from `input-event-codes.h` in
//! the Linux kernel.
//!
//! Some other operating systems may also use the same input codes as Linux, namely FreeBSD.
//!
//! These input codes may be used in conjunction with kernel apis to read input events or with a higher level library
//! such as [libinput](https://crates.io/crates/input).

#![no_std]

mod generated;

pub use self::generated::*;
