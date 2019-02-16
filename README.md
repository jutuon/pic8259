# pc-at-pic8259a

This is a driver for the Intel 8259A Programmable Interrupt Controller (PIC) as found in the IBM PC/AT
computer. In the PC/AT there are two PICs, master and slave, running in cascade mode.

## Example

```rust
use pc_at_pic8259a::*;

struct PicPortIO;
#[derive(Copy, Clone)]
struct PortID(u16);

impl PortIO for PicPortIO {
    type PortID = PortID;

    const MASTER_PIC_COMMAND_PORT: Self::PortID = PortID(MASTER_PIC_COMMAND_PORT_RAW);
    const MASTER_PIC_DATA_PORT: Self::PortID = PortID(MASTER_PIC_DATA_PORT_RAW);

    const SLAVE_PIC_COMMAND_PORT: Self::PortID = PortID(SLAVE_PIC_COMMAND_PORT_RAW);
    const SLAVE_PIC_DATA_PORT: Self::PortID = PortID(SLAVE_PIC_DATA_PORT_RAW);  

    fn read(&self, port: Self::PortID) -> u8 {
        unimplemented!() // unsafe { x86::io::inb(port.0) }
    }

    fn write(&mut self, port: Self::PortID, data: u8) {
        // unsafe { x86::io::outb(port.0, data); }
    }
}

const MASTER_PIC_INTERRUPT_OFFSET: u8 = 32;
const SLAVE_PIC_INTERRUPT_OFFSET: u8 = MASTER_PIC_INTERRUPT_OFFSET + 8;

fn main() {
    let _pic = PicInit::send_icw1(PicPortIO, InterruptTriggerMode::EdgeTriggered)
        .send_icw2_and_icw3(MASTER_PIC_INTERRUPT_OFFSET, SLAVE_PIC_INTERRUPT_OFFSET)
        .send_icw4_aeoi();

    // Setup Interrupt Descriptor Table...

    // Enable interrupts.
    // unsafe { x86::irq::enable(); }
}
```

## License

This project is licensed under terms of

* Apache 2.0 license or
* MIT license

at your opinion.
