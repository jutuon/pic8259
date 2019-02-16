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
//! the problem.
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

pub mod init;
pub mod raw;

pub use init::{PicInit, InterruptTriggerMode};

use raw::{OCW3ReadRegisterCommand, OCW2Commands};

pub trait PortIO {
    const MASTER_PIC_COMMAND_PORT: u16 = 0x20;
    const MASTER_PIC_DATA_PORT: u16 = 0x21;

    const SLAVE_PIC_COMMAND_PORT: u16 = 0xA0;
    const SLAVE_PIC_DATA_PORT: u16 = 0xA1;

    fn read(&self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}

trait PrivatePortIO {
    fn read(&self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}

impl <T: PortIO> PrivatePortIO for PortIOWrapper<T> {
    fn read(&self, port: u16) -> u8 {
        self.0.read(port)
    }
    fn write(&mut self, port: u16, data: u8) {
        self.0.write(port, data)
    }
}

/// Wrapper for PortIO implementer type to disallow
/// access to it after PIC initialization process is started.
pub struct PortIOWrapper<T: PortIO>(T);

pub trait PortIOAvailable<T: PortIO> {
    fn port_io(&self) -> &PortIOWrapper<T>;
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T>;
}

/// Automatic end of interrupt mode PIC.
pub struct PicAEOI<T: PortIO>(PortIOWrapper<T>);

impl <T: PortIO> PortIOAvailable<T> for PicAEOI<T> {
    fn port_io(&self) -> &PortIOWrapper<T> { &self.0 }
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T> { &mut self.0 }
}

// Normal end of interrupt mode PIC.
pub struct Pic<T: PortIO>(PortIOWrapper<T>);

impl <T: PortIO> PortIOAvailable<T> for Pic<T> {
    fn port_io(&self) -> &PortIOWrapper<T> { &self.0 }
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T> { &mut self.0 }
}

/// Send end of interrupt command.
pub trait SendEOI<T: PortIO>: PortIOAvailable<T> {
    fn send_eoi_to_master(&mut self) {
        self.port_io_mut().write(T::MASTER_PIC_COMMAND_PORT, OCW2Commands::NonSpecificEOI.bits());
    }

    fn send_eoi_to_slave(&mut self) {
        self.port_io_mut().write(T::SLAVE_PIC_COMMAND_PORT, OCW2Commands::NonSpecificEOI.bits());
    }
}

impl <T: PortIO> SendEOI<T> for Pic<T> {}
impl <T: PortIO> SendEOI<T> for RegisterReadModeIRR<T, Pic<T>> {}
impl <T: PortIO> SendEOI<T> for RegisterReadModeISR<T, Pic<T>> {}

/// Methods for changing interrupt masks.
///
/// Note that probably spurious IRQs may occur unless
/// you mask every PIC interrupt.
///
/// <https://wiki.osdev.org/8259_PIC#Spurious_IRQs>
pub trait PicMask<T: PortIO>: PortIOAvailable<T> {
    fn set_master_mask(&mut self, mask: u8) {
        self.port_io_mut().write(T::MASTER_PIC_DATA_PORT, mask);
    }

    fn set_slave_mask(&mut self, mask: u8) {
        self.port_io_mut().write(T::SLAVE_PIC_DATA_PORT, mask);
    }

    fn master_mask(&self) -> u8 {
        self.port_io().read(T::MASTER_PIC_DATA_PORT)
    }

    fn slave_mask(&self) -> u8 {
        self.port_io().read(T::SLAVE_PIC_DATA_PORT)
    }
}

impl <T: PortIO> PicMask<T> for PicAEOI<T> {}
impl <T: PortIO> PicMask<T> for Pic<T> {}
impl <T: PortIO, U: PortIOAvailable<T>> PicMask<T> for RegisterReadModeIRR<T, U> {}
impl <T: PortIO, U: PortIOAvailable<T>> PicMask<T> for RegisterReadModeISR<T, U> {}

use core::marker::PhantomData;

/// Read Interrupt Request Register (IRR).
pub struct RegisterReadModeIRR<T: PortIO, U: PortIOAvailable<T>>(PhantomData<T>, U);

impl <T: PortIO, U: PortIOAvailable<T>> PortIOAvailable<T> for RegisterReadModeIRR<T, U> {
    fn port_io(&self) -> &PortIOWrapper<T> { self.1.port_io() }
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T> { self.1.port_io_mut() }
}

unsafe impl <T: PortIO, U: PortIOAvailable<T>> LockedReadRegister<T> for RegisterReadModeIRR<T, U> {
    const REGISTER: OCW3ReadRegisterCommand = OCW3ReadRegisterCommand::InterruptRequest;
}

impl <T: PortIO, U: PortIOAvailable<T>> RegisterReadModeIRR<T, U> {
    fn new(mut pic: U) -> Self {
        pic.port_io_mut().write(T::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut().write(T::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeIRR(PhantomData, pic)
    }

    pub fn exit(self) -> U {
        self.1
    }
}

/// Read In Service Register (ISR).
pub struct RegisterReadModeISR<T: PortIO, U: PortIOAvailable<T>>(PhantomData<T>, U);

impl <T: PortIO, U: PortIOAvailable<T>> PortIOAvailable<T> for RegisterReadModeISR<T, U> {
    fn port_io(&self) -> &PortIOWrapper<T> { self.1.port_io() }
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T> { self.1.port_io_mut() }
}

unsafe impl <T: PortIO, U: PortIOAvailable<T>> LockedReadRegister<T> for RegisterReadModeISR<T, U> {
    const REGISTER: OCW3ReadRegisterCommand = OCW3ReadRegisterCommand::InService;
}

impl <T: PortIO, U: PortIOAvailable<T>> RegisterReadModeISR<T, U> {
    fn new(mut pic: U) -> Self {
        pic.port_io_mut().write(T::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut().write(T::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeISR(PhantomData, pic)
    }

    pub fn exit(self) -> U {
        self.1
    }
}

/// Implementer of this trait must set correct register read state.
pub unsafe trait LockedReadRegister<T: PortIO>: PortIOAvailable<T> {
    const REGISTER: OCW3ReadRegisterCommand;

    fn read_master(&self) -> u8 {
        self.port_io().read(T::MASTER_PIC_COMMAND_PORT)
    }

    fn read_slave(&self) -> u8 {
        self.port_io().read(T::SLAVE_PIC_COMMAND_PORT)
    }
}

/// Change PIC register read mode.
pub trait ChangeRegisterReadMode<T: PortIO>: Sized + PortIOAvailable<T> {
    fn read_irr_mode(self) -> RegisterReadModeIRR<T, Self> {
        RegisterReadModeIRR::new(self)
    }

    fn read_isr_mode(self) -> RegisterReadModeISR<T, Self> {
        RegisterReadModeISR::new(self)
    }
}

impl <T: PortIO> ChangeRegisterReadMode<T> for PicAEOI<T> {}
impl <T: PortIO> ChangeRegisterReadMode<T> for Pic<T> {}
