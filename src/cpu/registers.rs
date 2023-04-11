
#[derive(Debug)]
pub struct Registers {
    // 8-bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // 16-bit registers
    pub pc: u16,
    pub sp: u16,

    // Flags
    pub flag_z: bool,
    pub flag_n: bool,
    pub flag_h: bool,
    pub flag_c: bool,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0x0100,
            sp: 0xFFFE,
            flag_z: false,
            flag_n: false,
            flag_h: false,
            flag_c: false,
        }
    }

    // Get the value of the 16-bit register af
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f() as u16)
    }

    // Get the value of the 16-bit register bc
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    // Get the value of the 16-bit register de
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    // Get the value of the 16-bit register hl
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    // Get the value of the 8-bit flag register f
    pub fn f(&self) -> u8 {
        let mut f = 0;

        if self.flag_z {
            f |= 0b1000_0000;
        }

        if self.flag_n {
            f |= 0b0100_0000;
        }

        if self.flag_h {
            f |= 0b0010_0000;
        }

        if self.flag_c {
            f |= 0b0001_0000;
        }

        f
    }

    // Set the value of the 16-bit register af
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.set_f(value as u8);
    }

    // Set the value of the 16-bit register bc
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    // Set the value of the 16-bit register de
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    // Set the value of the 16-bit register hl
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    // Set the value of the 8-bit flag register f
    pub fn set_f(&mut self, value: u8) {
        self.flag_z = (value & 0b1000_0000) != 0;
        self.flag_n = (value & 0b0100_0000) != 0;
        self.flag_h = (value & 0b0010_0000) != 0;
        self.flag_c = (value & 0b0001_0000) != 0;
    }

    // Increment the program counter by n
    pub fn inc_pc(&mut self, n: u8) {
        self.pc += n as u16;
    }

    // Increment the stack pointer by n
    pub fn inc_sp(&mut self, n: u8) {
        self.sp += n as u16;
    }

    // Decrement the stack pointer by n
    pub fn dec_sp(&mut self, n: u8) {
        self.sp -= n as u16;
    }
}
