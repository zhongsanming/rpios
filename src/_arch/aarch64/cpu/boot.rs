//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path
//! attribute, the path of this file is:
//!
//! crate::cpu::boot::arch_boot;

// TODO: rust analyzer settings?

use core::arch::global_asm;

// Assembly counterpart to this file.
global_asm!(include_str!("boot.s"));
