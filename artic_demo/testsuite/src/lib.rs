#![no_std]
#![cfg_attr(test, no_main)]

use artic_demo as _; // memory layout + panic handler

#[defmt_test::tests]
mod tests {}
