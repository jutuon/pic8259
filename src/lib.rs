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
//! // TODO
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

#![no_std]

pub unsafe trait PortIO {
    const MASTER_PIC_COMMAND_PORT: u16 = 0x20;
    const MASTER_PIC_DATA_PORT: u16 = 0x21;

    const SLAVE_PIC_COMMAND_PORT: u16 = 0xA0;
    const SLAVE_PIC_DATA_PORT: u16 = 0xA1;

    fn read(&self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}

const PIC_INIT: u8 = 0b0001_0000;
const ENABLE_ICW4: u8 = 0b0000_0001;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
/// Available interrupt trigger modes.
///
/// Also contains other ICW1 bitflags.
pub enum InterruptTriggerMode {
    EdgeTriggered = PIC_INIT | ENABLE_ICW4,
    /// Level triggered mode is only used with IBM PS/2 computer.
    ///
    /// See section 7, page 1 (PDF page 262) from
    /// <http://classiccomputers.info/down/IBM_PS2/documents/PS2_Hardware_Interface_Technical_Reference_May88.pdf>
    LevelTriggered = 0b0000_1000 | PIC_INIT | ENABLE_ICW4,
}

/// Start master and slave PIC initialization.
///
/// PICs are initialized with four Initialization Command Words (ICW).
pub struct PicInit<T: PortIO>(T);

impl <T: PortIO> PicInit<T> {
    /// Send ICW1.
    pub fn send_icw1(mut port_io: T, mode: InterruptTriggerMode) -> ICW2AndICW3<T> {
        port_io.write(T::MASTER_PIC_COMMAND_PORT, mode as u8);
        port_io.write(T::SLAVE_PIC_COMMAND_PORT, mode as u8);

        ICW2AndICW3(port_io)
    }
}

/// Send the second and third Initialization Command Word (ICW).
pub struct ICW2AndICW3<T: PortIO>(T);

impl <T: PortIO> ICW2AndICW3<T> {
    /// Send ICW2 and ICW3.
    ///
    /// ICW2 sets interrupt number offset. ICW3 initializes cascade mode.
    ///
    /// # Panics
    ///
    /// * If `offset & 0b0000_0111 != 0`.
    pub fn send_icw2_and_icw3(mut self, master_offset: u8, slave_offset: u8) -> ICW4<T> {
        const NOT_USED_BITS_MASK: u8 = 0b0000_0111;

        if master_offset & NOT_USED_BITS_MASK != 0 {
            panic!("master_offset & {:#08b} != 0", NOT_USED_BITS_MASK);
        }

        if slave_offset & NOT_USED_BITS_MASK != 0 {
            panic!("slave_offset & {:#08b} != 0", NOT_USED_BITS_MASK);
        }

        self.0.write(T::MASTER_PIC_DATA_PORT, master_offset);
        self.0.write(T::SLAVE_PIC_DATA_PORT, slave_offset);

        // Send ICW3

        // Bit 2 means that slave is connected to master PIC's IRQ 2 line.
        const CONNECTED_SLAVES: u8 = 0b0000_0100;
        self.0.write(T::MASTER_PIC_DATA_PORT, CONNECTED_SLAVES);

        // IRQ line number where slave PIC is connected.
        const SLAVE_PIC_ID: u8 = 2;
        self.0.write(T::SLAVE_PIC_DATA_PORT, SLAVE_PIC_ID);

        ICW4(self.0)
    }
}


const ICW4_8068_MODE: u8 = 0b0000_0001;
const ICW4_AUTOMATIC_END_OF_INTERRUPT: u8 = 0b0000_0010;

pub struct ICW4<T: PortIO>(T);

impl <T: PortIO> ICW4<T> {
    /// Send ICW4 which sets PICs to Automatic End Of Interrupt (AEOI) mode.
    ///
    /// This is the recommended PIC mode, because you don't
    /// send end of interrupt message to PICs after every
    /// interrupt.
    pub fn send_icw4_aeoi(mut self) -> PicAEOI<T> {
        let icw4 = ICW4_8068_MODE | ICW4_AUTOMATIC_END_OF_INTERRUPT;
        self.0.write(T::MASTER_PIC_DATA_PORT, icw4);
        self.0.write(T::SLAVE_PIC_DATA_PORT, icw4);

        PicAEOI(self.0)
    }

    /// Send ICW4 which sets PICs to default End Of Interrupt (EOI) mode.
    ///
    /// In this mode you must send a end of interrupt
    /// message when receiving interrupt from PIC.
    pub fn send_icw4(mut self) -> Pic<T> {
        let icw4 = ICW4_8068_MODE;
        self.0.write(T::MASTER_PIC_DATA_PORT, icw4);
        self.0.write(T::SLAVE_PIC_DATA_PORT, icw4);

        Pic(self.0)
    }
}

pub struct PicAEOI<T: PortIO>(T);

impl <T: PortIO> PicAEOI<T> {
    pub fn read_irr_mode(self) -> RegisterReadModeIRR<Self, T> {
        RegisterReadModeIRR::new(self)
    }

    pub fn read_isr_mode(self) -> RegisterReadModeISR<Self, T> {
        RegisterReadModeISR::new(self)
    }
}

impl <T: PortIO> PicMask<T> for PicAEOI<T> {
    fn port_io(&self) -> &T { &self.0 }
    fn port_io_mut(&mut self) -> &mut T { &mut self.0 }
}

pub struct Pic<T: PortIO>(T);

impl <T: PortIO> Pic<T> {
    pub fn send_end_of_interrupt(&mut self) {
        const OCW2_NON_SPECIFIC_EOI: u8 = 0b0010_0000;
        self.port_io_mut().write(T::MASTER_PIC_COMMAND_PORT, OCW2_NON_SPECIFIC_EOI);
        self.port_io_mut().write(T::SLAVE_PIC_COMMAND_PORT, OCW2_NON_SPECIFIC_EOI);
    }

    pub fn read_irr_mode(self) -> RegisterReadModeIRR<Self, T> {
        RegisterReadModeIRR::new(self)
    }

    pub fn read_isr_mode(self) -> RegisterReadModeISR<Self, T> {
        RegisterReadModeISR::new(self)
    }
}

impl <T: PortIO> PicMask<T> for Pic<T> {
    fn port_io(&self) -> &T { &self.0 }
    fn port_io_mut(&mut self) -> &mut T { &mut self.0 }
}

/// Methods for changing interrupt masks.
///
/// Note that probably spurious IRQs may occur unless
/// you mask every PIC interrupt.
///
/// <https://wiki.osdev.org/8259_PIC#Spurious_IRQs>
pub trait PicMask<T: PortIO> {
    fn port_io(&self) -> &T;
    fn port_io_mut(&mut self) -> &mut T;

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

const OCW3_BASE_VALUE: u8 = 0b0000_1000;

#[derive(Debug)]
#[repr(u8)]
/// Registers which can be read from PIC.
pub enum Register {
    InterruptRequest = OCW3_BASE_VALUE | 0b0000_0010,
    InService = OCW3_BASE_VALUE | 0b0000_0011,
}

use core::marker::PhantomData;

pub struct RegisterReadModeIRR<T: PicMask<P>, P: PortIO>(T, PhantomData<P>);

impl <T: PicMask<P>, P: PortIO> RegisterReadModeIRR<T, P> {
    fn new(mut pic: T) -> Self {
        pic.port_io_mut().write(P::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut().write(P::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeIRR(pic, PhantomData)
    }

    pub fn pic(&self) -> &T {
        &self.0
    }

    pub fn pic_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn exit(self) -> T {
        self.0
    }
}

unsafe impl <T: PicMask<P>, P: PortIO> LockedReadRegister<P> for RegisterReadModeIRR<T, P> {
    const REGISTER: Register = Register::InterruptRequest;
    fn port_io(&self) -> &P { self.0.port_io() }
}

pub struct RegisterReadModeISR<T: PicMask<P>, P: PortIO>(T, PhantomData<P>);

impl <T: PicMask<P>, P: PortIO> RegisterReadModeISR<T, P> {
    fn new(mut pic: T) -> Self {
        pic.port_io_mut().write(P::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut().write(P::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeISR(pic, PhantomData)
    }

    pub fn pic(&self) -> &T {
        &self.0
    }

    pub fn pic_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn exit(self) -> T {
        self.0
    }
}

unsafe impl <T: PicMask<P>, P: PortIO> LockedReadRegister<P> for RegisterReadModeISR<T, P> {
    const REGISTER: Register = Register::InService;
    fn port_io(&self) -> &P { self.0.port_io() }
}


/// Implementer of this trait must set correct register read state.
pub unsafe trait LockedReadRegister<T: PortIO> {
    const REGISTER: Register;

    fn port_io(&self) -> &T;

    fn read_master(&self) -> u8 {
        self.port_io().read(T::MASTER_PIC_COMMAND_PORT)
    }

    fn read_slave(&self) -> u8 {
        self.port_io().read(T::SLAVE_PIC_COMMAND_PORT)
    }
}
