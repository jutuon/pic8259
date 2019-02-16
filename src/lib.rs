//! This is a driver for the Intel 8259A Programmable Interrupt Controller (PIC) as found in the IBM PC/AT
//! computer. In the PC/AT there are two PICs, master and slave, running in cascade mode. Current BIOS enabled PC computers
//! don't physically have 8259A chip inside, but the port IO interface still exists because of the
//! backwards compatibility.
//!
//! The PIC doesn't support multiple processor cores. Use APIC if you need support for those.
//! <https://en.wikipedia.org/wiki/Advanced_Programmable_Interrupt_Controller>
//!
//! Even if you use APIC this crate is still useful to you because according to
//! <https://wiki.osdev.org/APIC>, the PIC
//! must be disabled properly before using APIC.
//!
//! # Example
//!
//! ```
//! use pc_at_pic8259a::*;
//!
//! struct PicPortIO;
//!
//! impl PortIO for PicPortIO {
//!     fn read(&self, port: u16) -> u8 {
//!         unimplemented!() // unsafe { x86::io::inb(port) }
//!     }
//!
//!     fn write(&mut self, port: u16, data: u8) {
//!         // unsafe { x86::io::outb(port, data); }
//!     }
//! }
//!
//! const MASTER_PIC_INTERRUPT_OFFSET: u8 = 32;
//! const SLAVE_PIC_INTERRUPT_OFFSET: u8 = MASTER_PIC_INTERRUPT_OFFSET + 8;
//!
//! fn main() {
//!     let _pic = PicInit::send_icw1(PicPortIO, InterruptTriggerMode::EdgeTriggered)
//!         .send_icw2_and_icw3(MASTER_PIC_INTERRUPT_OFFSET, SLAVE_PIC_INTERRUPT_OFFSET)
//!         .send_icw4_aeoi();
//!
//!     // Setup Interrupt Descriptor Table...
//!
//!     // Enable interrupts.
//!     // unsafe { x86::irq::enable(); }
//! }
//!
//! ```
//!
//! # Hardware compatibility notes
//!
//! You may skip this section if you are
//! using newer hardware (Intel 486 or later).
//!
//! ## IO delays
//!
//! In OSDev Wiki reference code (<https://wiki.osdev.org/PIC#Initialisation>)
//! and IBM PC/AT BIOS reference code (<http://classiccomputers.info/down/IBM/IBM_AT_5170/IBM_5170_Technical_Reference_6280070_Sep85.pdf>,
//! Section 5, page 37, which is PDF page 209) does IO delay after writing to the PIC IO ports.
//! According to <https://forum.osdev.org/viewtopic.php?p=257289#p257289> this is not
//! required with modern hardware (Intel 486 or later) or when
//! PIC IO access is interleaved between master and slave PIC.
//!
//! When using old hardware you can add your own IO delay
//! code to `PortIO` trait implementation which should solve
//! the problem. Optionally read the source code of this library
//! and add delay to where it is necessary.
//!
//! ## Automatic end of interrupt (AEOI)
//!
//! The Intel reference for the PIC says the following:
//! > The AEOI mode can only be used in a master 8259A and not a slave.
//! > 8259As with a copyright date of 1985 or later will
//! > operate in the AEOI mode as a master or a slave.
//!
//! This library assumes that AEOI mode works with slave PIC.
//!
//! Some research about AEOI support:
//!
//! * <http://www.vcfed.org/forum/archive/index.php/t-50290.html>
//! * <https://scalibq.wordpress.com/2015/12/15/pc-compatibility-its-all-relative/>
//!
//! # Currently unimplemented features
//!
//! Read the Intel reference for more info about these features.
//!
//! * Specific End Of Interrupt
//! * Interrupt priority rotation
//! * Special fully nested mode
//!
//! # Why there is no option to enable PIC buffered mode?
//!
//! PC/AT probably doesn't require/support it, because IBM reference BIOS code
//! (<http://classiccomputers.info/down/IBM/IBM_AT_5170/IBM_5170_Technical_Reference_6280070_Sep85.pdf>,
//! Section 5, page 37, which is PDF page 209) doesn't enable it.
//!
//! # Reference material
//!
//! * Intel reference: <http://pdos.csail.mit.edu/6.828/2005/readings/hardware/8259A.pdf>
//!     * In the figures of the reference the A<sub>0</sub> = 0 is the
//!     command port and the A<sub>0</sub> = 1 is
//!     the data port.
//! * <https://wiki.osdev.org/8259_PIC>
//! * <https://en.wikipedia.org/wiki/Intel_8259>
//! * <https://wiki.osdev.org/User:Johnburger/PIC>
//! * <https://wiki.osdev.org/User:Johnburger/PIC/Background>

#![no_std]

pub mod raw;
pub mod io;
pub mod driver;

pub use driver::init::{PicInit, InterruptTriggerMode};
pub use io::PortIO;
