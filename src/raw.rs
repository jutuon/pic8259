//! Raw commands from the Intel reference.
//!
//! PC/AT doesn't support all commands in this module.

#[derive(Debug)]
pub struct ICW1Bits;

impl ICW1Bits {
    pub const ICW4_NEEDED: u8 = 0b0000_0001 | Self::ICW1_IDENTIFIER_BIT;
    pub const SINGLE_MODE: u8 = 0b0000_0010 | Self::ICW1_IDENTIFIER_BIT;
    pub const CALL_ADDRESS_INTERVAL_4: u8 = 0b0000_0100 | Self::ICW1_IDENTIFIER_BIT;
    pub const LEVEL_TRIGGERED_MODE: u8 = 0b0000_1000 | Self::ICW1_IDENTIFIER_BIT;
    pub const ICW1_IDENTIFIER_BIT: u8 = 0b0001_0000;
}

#[derive(Debug)]
pub struct ICW4Bits;

impl ICW4Bits {
    pub const ENABLE_8068_MODE: u8 = 0b0000_0001;
    pub const AUTOMATIC_END_OF_INTERRUPT: u8 = 0b0000_0010;
    pub const SPECIAL_FULLY_NESTED_MODE: u8 = 0b0001_0000;
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ICW4BufferedMode {
    Slave = 0b0000_1000,
    Master = 0b0000_1100,
}

impl ICW4BufferedMode {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OCW2Commands {
    NonSpecificEOI = 0b0010_0000,
    SpecificEOI = 0b0110_0000,
    RotateOnNonSpecificEOI = 0b1010_0000,
    RotateInAEOIModeSet = 0b1000_0000,
    RotateInAEOIModeClear = 0b0000_0000,
    /// Command requires `OCW2IRLevel`.
    RotateOnSpecificEOI = 0b1110_0000,
    /// Command requires `OCW2IRLevel`.
    SetPriority = 0b1100_0000,
    NoOperation = 0b0100_0000,
}

impl OCW2Commands {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OCW2IRLevel {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl OCW2IRLevel {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug)]
pub struct OCW3Bits;

impl OCW3Bits {
    pub const OCW3_IDENTIFIER_BIT: u8 = 0b0000_1000;
    pub const POLL_COMMAND: u8 = 0b0000_0100 | Self::OCW3_IDENTIFIER_BIT;
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OCW3SpecialMaskMode {
    NoAction = OCW3Bits::OCW3_IDENTIFIER_BIT,
    Reset = 0b0100_0000 | OCW3Bits::OCW3_IDENTIFIER_BIT,
    Set = 0b0110_0000 | OCW3Bits::OCW3_IDENTIFIER_BIT,
}

impl OCW3SpecialMaskMode {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OCW3ReadRegisterCommand {
    InterruptRequest = 0b0000_0010 | OCW3Bits::OCW3_IDENTIFIER_BIT,
    InService = 0b0000_0011 | OCW3Bits::OCW3_IDENTIFIER_BIT,
}

impl OCW3ReadRegisterCommand {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}
