//! Driver for PC/AT Programmable Interrupt Controllers.
//! 
//! # Reference material
//! 
//! * <http://pdos.csail.mit.edu/6.828/2005/readings/hardware/8259A.pdf>
//! * <https://wiki.osdev.org/8259_PIC>
//! * <https://en.wikipedia.org/wiki/Intel_8259>

#![no_std]

pub unsafe trait PortIO {
    // A0 = 0
    const MASTER_PIC_COMMAND_PORT: u16 = 0x20;
    // A0 = 1
    const MASTER_PIC_DATA_PORT: u16 = 0x21;

    // A0 = 0
    const SLAVE_PIC_COMMAND_PORT: u16 = 0xA0;
    // A0 = 1
    const SLAVE_PIC_DATA_PORT: u16 = 0xA1;

    fn read(&self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}

const PIC_INIT: u8 = 0b0001_0000;
const ENABLE_ICW4: u8 = 0b0000_0001;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
// This is used as ICW1
pub enum InterruptTriggerMode {
    EdgeTriggered = PIC_INIT | ENABLE_ICW4,
    LevelTriggered = 0b0000_1000 | PIC_INIT | ENABLE_ICW4,
}

pub struct PicInit<T: PortIO>(T);

impl <T: PortIO> PicInit<T> {
    pub fn start_init(mut port_io: T, mode: InterruptTriggerMode) -> ICW2<T> {
        port_io.write(T::MASTER_PIC_COMMAND_PORT, mode as u8);
        port_io.write(T::SLAVE_PIC_COMMAND_PORT, mode as u8);
        
        ICW2(port_io)
    }
}

pub struct ICW2<T: PortIO>(T);

impl <T: PortIO> ICW2<T> {
    /// Send ICW2 and ICW3.
    /// 
    /// Panics if `offset & 0b0000_0111 != 0`.
    pub fn interrupt_offsets(mut self, master_offset: u8, slave_offset: u8) -> ICW4<T> {
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
    pub fn automatic_end_of_interrupt(mut self) -> PicAEOI<T> {
        let icw4 = ICW4_8068_MODE | ICW4_AUTOMATIC_END_OF_INTERRUPT;
        self.0.write(T::MASTER_PIC_DATA_PORT, icw4);
        self.0.write(T::SLAVE_PIC_DATA_PORT, icw4);

        PicAEOI(self.0)
    }

    pub fn normal_end_of_interrupt(mut self) -> Pic<T> {
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


/// Implementor of this trait must set correct register read state.
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
