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

pub(crate) trait PrivatePortIO {
    type PortID;

    fn read(&self, port: Self::PortID) -> u8;
    fn write(&mut self, port: Self::PortID, data: u8);
}

impl <T: PortIO> PrivatePortIO for PortIOWrapper<T> {
    type PortID = T::PortID;

    fn read(&self, port: Self::PortID) -> u8 {
        self.0.read(port)
    }
    fn write(&mut self, port: Self::PortID, data: u8) {
        self.0.write(port, data)
    }
}

/// Wrapper for PortIO implementer type to disallow
/// access to it after PIC initialization process is started.
pub struct PortIOWrapper<T: PortIO>(pub(crate) T);

pub trait PortIOAvailable<T: PortIO> {
    fn port_io(&self) -> &PortIOWrapper<T>;
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T>;
}
