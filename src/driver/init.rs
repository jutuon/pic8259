//! Device initialization.
//!
//!

use super::{PortIO, Pic, PicAEOI, PortIOWrapper};

use crate::raw::{ICW1Bits, ICW4Bits};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
/// Available interrupt trigger modes.
///
/// Also contains other ICW1 bitflags.
pub enum InterruptTriggerMode {
    EdgeTriggered = ICW1Bits::ICW4_NEEDED,
    /// Level triggered mode is only used with IBM PS/2 computer.
    ///
    /// See section 7, page 1 (PDF page 262) from
    /// <http://classiccomputers.info/down/IBM_PS2/documents/PS2_Hardware_Interface_Technical_Reference_May88.pdf>
    LevelTriggered = ICW1Bits::LEVEL_TRIGGERED_MODE | ICW1Bits::ICW4_NEEDED,
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


pub struct ICW4<T: PortIO>(T);

impl <T: PortIO> ICW4<T> {
    /// Send ICW4 which sets PICs to Automatic End Of Interrupt (AEOI) mode.
    ///
    /// Note that some PC hardware doesn't support AEOI mode.
    /// 
    /// This is the most efficient PIC mode, because you don't
    /// send end of interrupt message to PICs after every
    /// interrupt.
    pub fn send_icw4_aeoi(mut self) -> PicAEOI<T> {
        let icw4 = ICW4Bits::ENABLE_8068_MODE | ICW4Bits::AUTOMATIC_END_OF_INTERRUPT;
        self.0.write(T::MASTER_PIC_DATA_PORT, icw4);
        self.0.write(T::SLAVE_PIC_DATA_PORT, icw4);

        PicAEOI(PortIOWrapper(self.0))
    }

    /// Send ICW4 which sets PICs to default End Of Interrupt (EOI) mode.
    ///
    /// In this mode you must send a end of interrupt
    /// message when receiving interrupt from PIC.
    pub fn send_icw4(mut self) -> Pic<T> {
        let icw4 = ICW4Bits::ENABLE_8068_MODE;
        self.0.write(T::MASTER_PIC_DATA_PORT, icw4);
        self.0.write(T::SLAVE_PIC_DATA_PORT, icw4);

        Pic(PortIOWrapper(self.0))
    }
}
