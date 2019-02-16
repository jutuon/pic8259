//! Read and write the PIC registers.

pub const MASTER_PIC_COMMAND_PORT_RAW: u16 = 0x20;
pub const MASTER_PIC_DATA_PORT_RAW: u16 = 0x21;

pub const SLAVE_PIC_COMMAND_PORT_RAW: u16 = 0xA0;
pub const SLAVE_PIC_DATA_PORT_RAW: u16 = 0xA1;

pub trait PortIO {
    type PortID;

    const MASTER_PIC_COMMAND_PORT: Self::PortID;
    const MASTER_PIC_DATA_PORT: Self::PortID;

    const SLAVE_PIC_COMMAND_PORT: Self::PortID;
    const SLAVE_PIC_DATA_PORT: Self::PortID;

    fn read(&self, port: Self::PortID) -> u8;
    fn write(&mut self, port: Self::PortID, data: u8);
}

/// You should only use this trait for debugging purposes.
pub trait PortIOAvailable<T: PortIO> {
    fn port_io(&self) -> &T;
    fn port_io_mut(&mut self) -> &mut T;
}
