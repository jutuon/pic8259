//! Read and write the PIC registers.

pub trait PortIO {
    const MASTER_PIC_COMMAND_PORT: u16 = 0x20;
    const MASTER_PIC_DATA_PORT: u16 = 0x21;

    const SLAVE_PIC_COMMAND_PORT: u16 = 0xA0;
    const SLAVE_PIC_DATA_PORT: u16 = 0xA1;

    fn read(&self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}

pub(crate) trait PrivatePortIO {
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
pub struct PortIOWrapper<T: PortIO>(pub(crate) T);

pub trait PortIOAvailable<T: PortIO> {
    fn port_io(&self) -> &PortIOWrapper<T>;
    fn port_io_mut(&mut self) -> &mut PortIOWrapper<T>;
}
