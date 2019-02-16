pub mod init;

use crate::io::*;

use crate::raw::{OCW2Commands, OCW3ReadRegisterCommand};

/// Automatic end of interrupt mode PIC.
pub struct PicAEOI<T: PortIO>(T);

impl_port_io_available!(<T: PortIO> PicAEOI<T>);

// Normal end of interrupt mode PIC.
pub struct Pic<T: PortIO>(T);

impl_port_io_available!(<T: PortIO> Pic<T>);

/// Send end of interrupt command.
pub trait SendEOI<T: PortIO>: PortIOAvailable<T> {
    fn send_eoi_to_master(&mut self) {
        self.port_io_mut().write(
            T::MASTER_PIC_COMMAND_PORT,
            OCW2Commands::NonSpecificEOI.bits(),
        );
    }

    fn send_eoi_to_slave_and_master(&mut self) {
        self.port_io_mut().write(
            T::SLAVE_PIC_COMMAND_PORT,
            OCW2Commands::NonSpecificEOI.bits(),
        );
        self.send_eoi_to_master();
    }
}

impl<T: PortIO> SendEOI<T> for Pic<T> {}
impl<T: PortIO> SendEOI<T> for RegisterReadModeIRR<T, Pic<T>> {}
impl<T: PortIO> SendEOI<T> for RegisterReadModeISR<T, Pic<T>> {}

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

impl<T: PortIO> PicMask<T> for PicAEOI<T> {}
impl<T: PortIO> PicMask<T> for Pic<T> {}
impl<T: PortIO, U: PortIOAvailable<T>> PicMask<T> for RegisterReadModeIRR<T, U> {}
impl<T: PortIO, U: PortIOAvailable<T>> PicMask<T> for RegisterReadModeISR<T, U> {}

use core::marker::PhantomData;

/// Read Interrupt Request Register (IRR).
pub struct RegisterReadModeIRR<T: PortIO, U: PortIOAvailable<T>>(PhantomData<T>, U);

impl_port_io_available!(<T: PortIO, U: PortIOAvailable<T>> RegisterReadModeIRR<T, U>);

impl<T: PortIO, U: PortIOAvailable<T>> LockedReadRegister<T> for RegisterReadModeIRR<T, U> {
    const REGISTER: OCW3ReadRegisterCommand = OCW3ReadRegisterCommand::InterruptRequest;
}

impl<T: PortIO, U: PortIOAvailable<T>> RegisterReadModeIRR<T, U> {
    fn new(mut pic: U) -> Self {
        pic.port_io_mut()
            .write(T::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut()
            .write(T::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeIRR(PhantomData, pic)
    }

    pub fn exit(self) -> U {
        self.1
    }
}

/// Read In Service Register (ISR).
pub struct RegisterReadModeISR<T: PortIO, U: PortIOAvailable<T>>(PhantomData<T>, U);

impl_port_io_available!(<T: PortIO, U: PortIOAvailable<T>> RegisterReadModeISR<T, U>);

impl<T: PortIO, U: PortIOAvailable<T>> LockedReadRegister<T> for RegisterReadModeISR<T, U> {
    const REGISTER: OCW3ReadRegisterCommand = OCW3ReadRegisterCommand::InService;
}

impl<T: PortIO, U: PortIOAvailable<T>> RegisterReadModeISR<T, U> {
    fn new(mut pic: U) -> Self {
        pic.port_io_mut()
            .write(T::MASTER_PIC_COMMAND_PORT, Self::REGISTER as u8);
        pic.port_io_mut()
            .write(T::SLAVE_PIC_COMMAND_PORT, Self::REGISTER as u8);
        RegisterReadModeISR(PhantomData, pic)
    }

    pub fn exit(self) -> U {
        self.1
    }
}

/// Implementer of this trait must set correct register read state.
pub trait LockedReadRegister<T: PortIO>: PortIOAvailable<T> {
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

impl<T: PortIO> ChangeRegisterReadMode<T> for PicAEOI<T> {}
impl<T: PortIO> ChangeRegisterReadMode<T> for Pic<T> {}
