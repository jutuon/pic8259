# pc-at-pic8259a

This is a driver for the Intel 8259A Programmable Interrupt Controller (PIC) as found in the IBM PC/AT
computer. In the PC/AT there are two PICs, master and slave, running in cascade mode.

## Example

```rust
use pc_at_pic8259a::*;

struct PicPortIO;

impl PortIO for PicPortIO {
    fn read(&self, port: u16) -> u8 {
        unimplemented!() // unsafe { x86::io::inb(port) }
    }

    fn write(&mut self, port: u16, data: u8) {
        // unsafe { x86::io::outb(port, data); }
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
