//! Read and write the PIC registers.

pub const MASTER_PIC_COMMAND_PORT_RAW: u16 = 0x20;
pub const MASTER_PIC_DATA_PORT_RAW: u16 = 0x21;

pub const SLAVE_PIC_COMMAND_PORT_RAW: u16 = 0xA0;
pub const SLAVE_PIC_DATA_PORT_RAW: u16 = 0xA1;

pub trait PortIO {
    type PortID: Copy;

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

macro_rules! impl_port_io_available {
    (<T: PortIO> $type:ty) => {
        impl<T: PortIO> crate::io::PortIOAvailable<T> for $type {
            fn port_io(&self) -> &T {
                &self.0
            }
            fn port_io_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }
    };
    (<T: PortIO, U: PortIOAvailable<T>> $type:ty) => {
        impl<T: PortIO, U: PortIOAvailable<T>> crate::io::PortIOAvailable<T> for $type {
            fn port_io(&self) -> &T {
                self.1.port_io()
            }
            fn port_io_mut(&mut self) -> &mut T {
                self.1.port_io_mut()
            }
        }
    };
}
