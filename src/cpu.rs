mod registers;
use registers::Registers;

mod memory;
use memory::Memory;

mod instructions;
use instructions::{
    ADDHLTarget, ArithmeticTarget, IncDecTarget, Indirect, Instruction, JumpTest, LoadByteSource,
    LoadByteTarget, LoadType, LoadWordTarget, StackTarget,
};

use self::instructions::{BitPosition, PrefixTarget};

pub struct Cpu {
    pub registers: Registers,
    pub memory: Memory,
    pub is_halted: bool,
    pub interrupts_enabled: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            memory: Memory::new(),
            is_halted: false,
            interrupts_enabled: false,
        }
    }

    fn jump(&self, jump_condition: bool) -> (u16, u8) {
        if jump_condition {
            (self.read_next_word(), 16)
        } else {
            // If we don't jump we need to still move the program
            // counter forward by 3 since the jump instruction is
            // 3 bytes wide (1 byte for tag and 2 bytes for jump address)
            (self.registers.pc.wrapping_add(3), 12)
        }
    }

    fn jump_relative(&self, should_jump: bool) -> (u16, u8) {
        let next_step = self.registers.pc.wrapping_add(2);
        if should_jump {
            let offset = self.read_next_byte() as i8;
            let pc = if offset >= 0 {
                next_step.wrapping_add(offset as u16)
            } else {
                next_step.wrapping_sub(offset.abs() as u16)
            };
            (pc, 16)
        } else {
            (next_step, 12)
        }
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory
            .write_byte(self.registers.sp, ((value & 0xFF00) >> 8) as u8);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory
            .write_byte(self.registers.sp, (value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lsb = self.memory.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let msb = self.memory.read_byte(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    fn call(&mut self, condition: bool) -> (u16, u8) {
        let next_pc = self.registers.pc.wrapping_add(3);
        if condition {
            self.push(next_pc);
            (self.read_next_word(), 24)
        } else {
            (next_pc, 12)
        }
    }

    fn ret(&mut self, condition: bool) -> u16 {
        if condition {
            self.pop()
        } else {
            self.registers.pc.wrapping_add(1)
        }
    }

    fn rst(&mut self) {
        self.push(self.registers.pc.wrapping_add(1));
    }

    pub fn read_next_byte(&self) -> u8 {
        self.memory.read_byte(self.registers.pc + 1)
    }

    pub fn read_next_word(&self) -> u16 {
        // Gameboy is little endian so read pc + 2 as most significant bit
        // and pc + 1 as least significant bi
        let least_significant_byte = self.memory.read_byte(self.registers.pc + 1) as u16;
        let most_significant_byte = self.memory.read_byte(self.registers.pc + 2) as u16;
        (most_significant_byte << 8) | least_significant_byte
    }

    pub fn step(&mut self) {
        let op_byte = self.memory.read_byte(self.registers.pc);
        let prefixed = op_byte == 0xCB;
        if prefixed {
            let op_byte = self.memory.read_byte(self.registers.pc + 1);
        }
        // // TODO find out when to use prefixed opcodes and when not.
        // let instruction = Instruction::from_byte(op_byte, false).expect("Invalid opcode");
        // self.registers.pc += 1;
        // instruction.execute(self);

        let (new_pc, cycles) = if let Some(instruction) = Instruction::from_byte(op_byte, prefixed)
        {
            let description = format!("0x{}{:x} -> {:?}", if prefixed { "cb" } else { "" }, op_byte, instruction);
            println!("Stepped: {}", description);
            self.execute(instruction)
        } else {
            let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, op_byte);
            panic!("Unkown instruction found for: {}", description)
        };

        self.registers.pc = new_pc;
    }

    pub fn execute(&mut self, instruction: Instruction) -> (u16, u8) {
        match instruction {
            Instruction::ADD(target) => {
                match target {
                    ArithmeticTarget::A => {
                        self.registers.a = alu_add(self, self.registers.a, false)
                    }
                    ArithmeticTarget::B => {
                        self.registers.a = alu_add(self, self.registers.b, false)
                    }
                    ArithmeticTarget::C => {
                        self.registers.a = alu_add(self, self.registers.c, false)
                    }
                    ArithmeticTarget::D => {
                        self.registers.a = alu_add(self, self.registers.d, false)
                    }
                    ArithmeticTarget::E => {
                        self.registers.a = alu_add(self, self.registers.e, false)
                    }
                    ArithmeticTarget::H => {
                        self.registers.a = alu_add(self, self.registers.h, false)
                    }
                    ArithmeticTarget::L => {
                        self.registers.a = alu_add(self, self.registers.l, false)
                    }
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a = alu_add(self, value, false);
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a = alu_add(self, value, false);
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::ADC(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a = alu_add(self, self.registers.a, true),
                    ArithmeticTarget::B => self.registers.a = alu_add(self, self.registers.b, true),
                    ArithmeticTarget::C => self.registers.a = alu_add(self, self.registers.c, true),
                    ArithmeticTarget::D => self.registers.a = alu_add(self, self.registers.d, true),
                    ArithmeticTarget::E => self.registers.a = alu_add(self, self.registers.e, true),
                    ArithmeticTarget::H => self.registers.a = alu_add(self, self.registers.h, true),
                    ArithmeticTarget::L => self.registers.a = alu_add(self, self.registers.l, true),
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a = alu_add(self, value, true);
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a = alu_add(self, value, true);
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::SUB(target) => {
                match target {
                    ArithmeticTarget::A => {
                        self.registers.a = alu_sub(self, self.registers.a, false)
                    }
                    ArithmeticTarget::B => {
                        self.registers.a = alu_sub(self, self.registers.b, false)
                    }
                    ArithmeticTarget::C => {
                        self.registers.a = alu_sub(self, self.registers.c, false)
                    }
                    ArithmeticTarget::D => {
                        self.registers.a = alu_sub(self, self.registers.d, false)
                    }
                    ArithmeticTarget::E => {
                        self.registers.a = alu_sub(self, self.registers.e, false)
                    }
                    ArithmeticTarget::H => {
                        self.registers.a = alu_sub(self, self.registers.h, false)
                    }
                    ArithmeticTarget::L => {
                        self.registers.a = alu_sub(self, self.registers.l, false)
                    }
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a = alu_sub(self, value, false);
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a = alu_sub(self, value, false);
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::SBC(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a = alu_sub(self, self.registers.a, true),
                    ArithmeticTarget::B => self.registers.a = alu_sub(self, self.registers.b, true),
                    ArithmeticTarget::C => self.registers.a = alu_sub(self, self.registers.c, true),
                    ArithmeticTarget::D => self.registers.a = alu_sub(self, self.registers.d, true),
                    ArithmeticTarget::E => self.registers.a = alu_sub(self, self.registers.e, true),
                    ArithmeticTarget::H => self.registers.a = alu_sub(self, self.registers.h, true),
                    ArithmeticTarget::L => self.registers.a = alu_sub(self, self.registers.l, true),
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a = alu_sub(self, value, true);
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a = alu_sub(self, value, true);
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::AND(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a &= self.registers.a,
                    ArithmeticTarget::B => self.registers.a &= self.registers.b,
                    ArithmeticTarget::C => self.registers.a &= self.registers.c,
                    ArithmeticTarget::D => self.registers.a &= self.registers.d,
                    ArithmeticTarget::E => self.registers.a &= self.registers.e,
                    ArithmeticTarget::H => self.registers.a &= self.registers.h,
                    ArithmeticTarget::L => self.registers.a &= self.registers.l,
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a &= value;
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a &= value;
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::XOR(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a ^= self.registers.a,
                    ArithmeticTarget::B => self.registers.a ^= self.registers.b,
                    ArithmeticTarget::C => self.registers.a ^= self.registers.c,
                    ArithmeticTarget::D => self.registers.a ^= self.registers.d,
                    ArithmeticTarget::E => self.registers.a ^= self.registers.e,
                    ArithmeticTarget::H => self.registers.a ^= self.registers.h,
                    ArithmeticTarget::L => self.registers.a ^= self.registers.l,
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a ^= value;
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a ^= value;
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::OR(target) => {
                match target {
                    ArithmeticTarget::A => self.registers.a |= self.registers.a,
                    ArithmeticTarget::B => self.registers.a |= self.registers.b,
                    ArithmeticTarget::C => self.registers.a |= self.registers.c,
                    ArithmeticTarget::D => self.registers.a |= self.registers.d,
                    ArithmeticTarget::E => self.registers.a |= self.registers.e,
                    ArithmeticTarget::H => self.registers.a |= self.registers.h,
                    ArithmeticTarget::L => self.registers.a |= self.registers.l,
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        self.registers.a |= value;
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        self.registers.a |= value;
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::CP(target) => {
                match target {
                    ArithmeticTarget::A => _ = alu_sub(self, self.registers.a, false),
                    ArithmeticTarget::B => _ = alu_sub(self, self.registers.b, false),
                    ArithmeticTarget::C => _ = alu_sub(self, self.registers.c, false),
                    ArithmeticTarget::D => _ = alu_sub(self, self.registers.d, false),
                    ArithmeticTarget::E => _ = alu_sub(self, self.registers.e, false),
                    ArithmeticTarget::H => _ = alu_sub(self, self.registers.h, false),
                    ArithmeticTarget::L => _ = alu_sub(self, self.registers.l, false),
                    ArithmeticTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        _ = alu_sub(self, value, false);
                    }
                    ArithmeticTarget::D8 => {
                        let value = self.read_next_byte();
                        _ = alu_sub(self, value, false);
                    }
                }
                let (pc, cycles) = match target {
                    ArithmeticTarget::HLI => (1, 8),
                    ArithmeticTarget::D8 => (2, 8),
                    _ => (1, 4),
                };
                (self.registers.pc.wrapping_add(pc), cycles)
            }
            Instruction::INC(target) => {
                match target {
                    IncDecTarget::A => self.registers.a = inc_8bit(self, self.registers.a),
                    IncDecTarget::B => self.registers.b = inc_8bit(self, self.registers.b),
                    IncDecTarget::C => self.registers.c = inc_8bit(self, self.registers.c),
                    IncDecTarget::D => self.registers.d = inc_8bit(self, self.registers.d),
                    IncDecTarget::E => self.registers.e = inc_8bit(self, self.registers.e),
                    IncDecTarget::H => self.registers.h = inc_8bit(self, self.registers.h),
                    IncDecTarget::L => self.registers.l = inc_8bit(self, self.registers.l),
                    IncDecTarget::HLI => {
                        let hl = self.registers.hl();
                        let amount = self.memory.read_byte(hl);
                        let result = inc_8bit(self, amount);
                        self.memory.write_byte(hl, result);
                    }
                    IncDecTarget::BC => {
                        let value = self.registers.bc().wrapping_add(1);
                        self.registers.set_bc(value);
                    }
                    IncDecTarget::DE => {
                        let value = self.registers.de().wrapping_add(1);
                        self.registers.set_de(value);
                    }
                    IncDecTarget::HL => {
                        let value = self.registers.hl().wrapping_add(1);
                        self.registers.set_hl(value);
                    }
                    IncDecTarget::SP => {
                        let amount = self.registers.sp;
                        let result = self.registers.sp.wrapping_add(amount);
                        self.registers.sp = result;
                    }
                }
                let cycles = match target {
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => 8,
                    IncDecTarget::HLI => 12,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(1), cycles)
            }
            Instruction::DEC(target) => {
                match target {
                    IncDecTarget::A => self.registers.a = dec_8bit(self, self.registers.a),
                    IncDecTarget::B => self.registers.b = dec_8bit(self, self.registers.b),
                    IncDecTarget::C => self.registers.c = dec_8bit(self, self.registers.c),
                    IncDecTarget::D => self.registers.d = dec_8bit(self, self.registers.d),
                    IncDecTarget::E => self.registers.e = dec_8bit(self, self.registers.e),
                    IncDecTarget::H => self.registers.h = dec_8bit(self, self.registers.h),
                    IncDecTarget::L => self.registers.l = dec_8bit(self, self.registers.l),
                    IncDecTarget::HLI => {
                        let hl = self.registers.hl();
                        let amount = self.memory.read_byte(hl);
                        let result = dec_8bit(self, amount);
                        self.memory.write_byte(hl, result);
                    }
                    IncDecTarget::BC => {
                        let value = self.registers.bc().wrapping_sub(1);
                        self.registers.set_bc(value);
                    }
                    IncDecTarget::DE => {
                        let value = self.registers.de().wrapping_sub(1);
                        self.registers.set_de(value);
                    }
                    IncDecTarget::HL => {
                        let value = self.registers.hl().wrapping_sub(1);
                        self.registers.set_hl(value);
                    }
                    IncDecTarget::SP => {
                        let amount = self.registers.sp;
                        let result = self.registers.sp.wrapping_sub(amount);
                        self.registers.sp = result;
                    }
                }
                let cycles = match target {
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => 8,
                    IncDecTarget::HLI => 12,
                    _ => 4,
                };
                (self.registers.pc.wrapping_add(1), cycles)
            }
            Instruction::DAA => {
                self.registers.a = decimal_adjust(self, self.registers.a);
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::CPL => {
                self.registers.a = !self.registers.a;
                self.registers.flag_n = true;
                self.registers.flag_h = true;
                (self.registers.pc.wrapping_add(1), 4)
            }

            Instruction::ADDHL(target) => {
                match target {
                    ADDHLTarget::BC => {
                        let value = add_hl(self, self.registers.bc());
                        self.registers.set_hl(value);
                    }
                    ADDHLTarget::DE => {
                        let value = add_hl(self, self.registers.de());
                        self.registers.set_hl(value);
                    }
                    ADDHLTarget::HL => {
                        let value = add_hl(self, self.registers.hl());
                        self.registers.set_hl(value);
                    }
                    ADDHLTarget::SP => {
                        let value = add_hl(self, self.registers.hl());
                        self.registers.sp = value;
                    }
                }
                (self.registers.pc.wrapping_add(1), 8)
            }
            Instruction::ADDSP => {
                // DESCRIPTION: (add stack pointer) - add a one byte signed number to
                // the value stored in the stack pointer register
                // PC:+2
                // Cycles: 16
                // Z:0 S:0 H:? C:?

                // First cast the byte as signed with `as i8` then extend it to 16 bits
                // with `as i16` and then stop treating it like a signed integer with
                // `as u16`
                let value = self.read_next_byte() as i8 as i16 as u16;
                let result = self.registers.sp.wrapping_add(value);

                // Half and whole carry are computed at the nibble and byte level instead
                // of the byte and word level like you might expect for 16 bit values
                let half_carry_mask = 0xF;
                self.registers.flag_h = (self.registers.sp & half_carry_mask)
                    + (value & half_carry_mask)
                    > half_carry_mask;
                let carry_mask = 0xff;
                self.registers.flag_c =
                    (self.registers.sp & carry_mask) + (value & carry_mask) > carry_mask;
                self.registers.flag_z = false;
                self.registers.flag_n = false;

                self.registers.sp = result;

                (self.registers.pc.wrapping_add(2), 16)
            }

            Instruction::CCF => {
                self.registers.flag_n = false;
                self.registers.flag_h = false;
                self.registers.flag_c = !self.registers.flag_c;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::SCF => {
                // DESCRIPTION: (set carry flag) - set the carry flag to true
                // PC:+1
                // Cycles: 4
                // Z:- S:0 H:0 C:1
                self.registers.flag_n = false;
                self.registers.flag_h = false;
                self.registers.flag_c = true;
                (self.registers.pc.wrapping_add(1), 4)
            }

            Instruction::LD(load_type) => {
                match load_type {
                    // DESCRIPTION: load byte store in a particular register into another
                    // particular register
                    // WHEN: source is d8
                    // PC:+2
                    // Cycles: 8
                    // WHEN: source is (HL)
                    // PC:+1
                    // Cycles: 8
                    // ELSE:
                    // PC:+1
                    // Cycles: 4
                    // Z:- N:- H:- C:-
                    LoadType::Byte(target, source) => {
                        let source_value = match source {
                            LoadByteSource::A => self.registers.a,
                            LoadByteSource::B => self.registers.b,
                            LoadByteSource::C => self.registers.c,
                            LoadByteSource::D => self.registers.d,
                            LoadByteSource::E => self.registers.e,
                            LoadByteSource::H => self.registers.h,
                            LoadByteSource::L => self.registers.l,
                            LoadByteSource::D8 => self.read_next_byte(),
                            LoadByteSource::HLI => self.memory.read_byte(self.registers.hl()),
                        };
                        match target {
                            LoadByteTarget::A => self.registers.a = source_value,
                            LoadByteTarget::B => self.registers.b = source_value,
                            LoadByteTarget::C => self.registers.c = source_value,
                            LoadByteTarget::D => self.registers.d = source_value,
                            LoadByteTarget::E => self.registers.e = source_value,
                            LoadByteTarget::H => self.registers.h = source_value,
                            LoadByteTarget::L => self.registers.l = source_value,
                            LoadByteTarget::HLI => {
                                self.memory.write_byte(self.registers.hl(), source_value)
                            }
                        };
                        match source {
                            LoadByteSource::D8 => (self.registers.pc.wrapping_add(2), 8),
                            LoadByteSource::HLI => (self.registers.pc.wrapping_add(1), 8),
                            _ => (self.registers.pc.wrapping_add(1), 4),
                        }
                    }
                    // DESCRIPTION: load next word in memory into a particular register
                    // PC:+3
                    // Cycles: 12
                    // Z:- N:- H:- C:-
                    LoadType::Word(target) => {
                        let word = self.read_next_word();
                        match target {
                            LoadWordTarget::BC => self.registers.set_bc(word),
                            LoadWordTarget::DE => self.registers.set_de(word),
                            LoadWordTarget::HL => self.registers.set_hl(word),
                            LoadWordTarget::SP => self.registers.sp = word,
                        };
                        (self.registers.pc.wrapping_add(3), 12)
                    }
                    // DESCRIPTION: load a particular value stored at the source address into A
                    // WHEN: source is word indirect
                    // PC:+3
                    // Cycles: 16
                    // ELSE:
                    // PC:+1
                    // Cycles: 8
                    // Z:- N:- H:- C:-
                    LoadType::AFromIndirect(source) => {
                        self.registers.a = match source {
                            Indirect::BCIndirect => self.memory.read_byte(self.registers.bc()),
                            Indirect::DEIndirect => self.memory.read_byte(self.registers.de()),
                            Indirect::HLIndirectMinus => {
                                let hl = self.registers.hl();
                                self.registers.set_hl(hl.wrapping_sub(1));
                                self.memory.read_byte(hl)
                            }
                            Indirect::HLIndirectPlus => {
                                let hl = self.registers.hl();
                                self.registers.set_hl(hl.wrapping_add(1));
                                self.memory.read_byte(hl)
                            }
                            Indirect::WordIndirect => self.memory.read_byte(self.read_next_word()),
                            Indirect::LastByteIndirect => {
                                self.memory.read_byte(0xFF00 + self.registers.c as u16)
                            }
                        };

                        match source {
                            Indirect::WordIndirect => (self.registers.pc.wrapping_add(3), 16),
                            _ => (self.registers.pc.wrapping_add(1), 8),
                        }
                    }
                    // DESCRIPTION: load the A register into memory at the source address
                    // WHEN: instruction.source is word indirect
                    // PC:+3
                    // Cycles: 16
                    // ELSE:
                    // PC:+1
                    // Cycles: 8
                    // Z:- N:- H:- C:-
                    LoadType::IndirectFromA(target) => {
                        let a = self.registers.a;
                        match target {
                            Indirect::BCIndirect => {
                                let bc = self.registers.bc();
                                self.memory.write_byte(bc, a)
                            }
                            Indirect::DEIndirect => {
                                let de = self.registers.de();
                                self.memory.write_byte(de, a)
                            }
                            Indirect::HLIndirectMinus => {
                                let hl = self.registers.hl();
                                self.registers.set_hl(hl.wrapping_sub(1));
                                self.memory.write_byte(hl, a);
                            }
                            Indirect::HLIndirectPlus => {
                                let hl = self.registers.hl();
                                self.registers.set_hl(hl.wrapping_add(1));
                                self.memory.write_byte(hl, a);
                            }
                            Indirect::WordIndirect => {
                                let word = self.read_next_word();
                                self.memory.write_byte(word, a);
                            }
                            Indirect::LastByteIndirect => {
                                let c = self.registers.c as u16;
                                self.memory.write_byte(0xFF00 + c, a);
                            }
                        };

                        match target {
                            Indirect::WordIndirect => (self.registers.pc.wrapping_add(3), 16),
                            _ => (self.registers.pc.wrapping_add(1), 8),
                        }
                    }
                    // DESCRIPTION: Load the value in A into memory location located at 0xFF plus
                    // an offset stored as the next byte in memory
                    // PC:+2
                    // Cycles: 12
                    // Z:- N:- H:- C:-
                    LoadType::ByteAddressFromA => {
                        let offset = self.read_next_byte() as u16;
                        self.memory.write_byte(0xFF00 + offset, self.registers.a);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    // DESCRIPTION: Load the value located at 0xFF plus an offset stored as the next byte in memory into A
                    // PC:+2
                    // Cycles: 12
                    // Z:- N:- H:- C:-
                    LoadType::AFromByteAddress => {
                        let offset = self.read_next_byte() as u16;
                        self.registers.a = self.memory.read_byte(0xFF00 + offset);
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                    // DESCRIPTION: Load the value in HL into SP
                    // PC:+1
                    // Cycles: 8
                    // Z:- N:- H:- C:-
                    LoadType::SPFromHL => {
                        self.registers.sp = self.registers.hl();
                        (self.registers.pc.wrapping_add(1), 8)
                    }
                    // DESCRIPTION: Load memory address with the contents of SP
                    // PC:+3
                    // Cycles: 20
                    // Z:- N:- H:- C:-
                    LoadType::IndirectFromSP => {
                        let address = self.read_next_word();
                        let sp = self.registers.sp;
                        self.memory.write_byte(address, (sp & 0xFF) as u8);
                        self.memory
                            .write_byte(address.wrapping_add(1), ((sp & 0xFF00) >> 8) as u8);
                        (self.registers.pc.wrapping_add(3), 20)
                    }
                    // DESCRIPTION: load HL with SP plus some specified byte
                    // PC:+2
                    // Cycles: 12
                    // Z:0 N:0 H:? C:?
                    LoadType::HLFromSPN => {
                        let value = self.read_next_byte() as i8 as i16 as u16;
                        let result = self.registers.sp.wrapping_add(value);
                        self.registers.set_hl(result);
                        self.registers.flag_z = false;
                        self.registers.flag_n = false;
                        // Half and whole carry are computed at the nibble and byte level instead
                        // of the byte and word level like you might expect for 16 bit values
                        self.registers.flag_h = (self.registers.sp & 0xF) + (value & 0xF) > 0xF;
                        self.registers.flag_c = (self.registers.sp & 0xFF) + (value & 0xFF) > 0xFF;
                        (self.registers.pc.wrapping_add(2), 12)
                    }
                }
            }

            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.flag_z,
                    JumpTest::NotCarry => !self.registers.flag_c,
                    JumpTest::Zero => self.registers.flag_z,
                    JumpTest::Carry => self.registers.flag_c,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::JR(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.flag_z,
                    JumpTest::NotCarry => !self.registers.flag_c,
                    JumpTest::Zero => self.registers.flag_z,
                    JumpTest::Carry => self.registers.flag_c,
                    JumpTest::Always => true,
                };
                self.jump_relative(jump_condition)
            }
            Instruction::JPI => (self.registers.hl(), 4),

            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.af(),
                    StackTarget::BC => self.registers.bc(),
                    StackTarget::DE => self.registers.de(),
                    StackTarget::HL => self.registers.hl(),
                };
                self.push(value);
                (self.registers.pc.wrapping_add(1), 16)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::AF => self.registers.set_af(result),
                    StackTarget::BC => self.registers.set_bc(result),
                    StackTarget::DE => self.registers.set_de(result),
                    StackTarget::HL => self.registers.set_hl(result),
                };
                (self.registers.pc.wrapping_add(1), 12)
            }
            Instruction::CALL(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.flag_z,
                    JumpTest::NotCarry => !self.registers.flag_c,
                    JumpTest::Zero => self.registers.flag_z,
                    JumpTest::Carry => self.registers.flag_c,
                    JumpTest::Always => true,
                };
                self.call(jump_condition)
            }
            Instruction::RET(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.flag_z,
                    JumpTest::NotCarry => !self.registers.flag_c,
                    JumpTest::Zero => self.registers.flag_z,
                    JumpTest::Carry => self.registers.flag_c,
                    JumpTest::Always => true,
                };
                let next_pc = self.ret(jump_condition);

                let cycles = if jump_condition && test == JumpTest::Always {
                    16
                } else if jump_condition {
                    20
                } else {
                    8
                };
                (next_pc, cycles)
            }

            Instruction::RRA => {
                self.registers.a = rotate_right_through_carry(self, self.registers.a, false);
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RLA => {
                self.registers.a = rotate_left_through_carry(self, self.registers.a, false);
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RRCA => {
                self.registers.a = rotate_right(self, self.registers.a, false);
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::RLCA => {
                self.registers.a = rotate_left(self, self.registers.a, false);
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::BIT(prefix, bit_position) => {
                let register = match prefix {
                    PrefixTarget::A => self.registers.a,
                    PrefixTarget::B => self.registers.b,
                    PrefixTarget::C => self.registers.c,
                    PrefixTarget::D => self.registers.d,
                    PrefixTarget::E => self.registers.e,
                    PrefixTarget::H => self.registers.h,
                    PrefixTarget::L => self.registers.l,
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        self.memory.read_byte(address)
                    }
                };
                bit_test(self, register, bit_position);

                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::RES(prefix, bit_position) => {
                match prefix {
                    PrefixTarget::A => self.registers.a = reset_bit(self.registers.a, bit_position),
                    PrefixTarget::B => self.registers.b = reset_bit(self.registers.b, bit_position),
                    PrefixTarget::C => self.registers.c = reset_bit(self.registers.c, bit_position),
                    PrefixTarget::D => self.registers.d = reset_bit(self.registers.d, bit_position),
                    PrefixTarget::E => self.registers.e = reset_bit(self.registers.e, bit_position),
                    PrefixTarget::H => self.registers.h = reset_bit(self.registers.h, bit_position),
                    PrefixTarget::L => self.registers.l = reset_bit(self.registers.l, bit_position),
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = reset_bit(value, bit_position);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::SET(prefix, bit_position) => {
                match prefix {
                    PrefixTarget::A => self.registers.a = set_bit(self.registers.a, bit_position),
                    PrefixTarget::B => self.registers.b = set_bit(self.registers.b, bit_position),
                    PrefixTarget::C => self.registers.c = set_bit(self.registers.c, bit_position),
                    PrefixTarget::D => self.registers.d = set_bit(self.registers.d, bit_position),
                    PrefixTarget::E => self.registers.e = set_bit(self.registers.e, bit_position),
                    PrefixTarget::H => self.registers.h = set_bit(self.registers.h, bit_position),
                    PrefixTarget::L => self.registers.l = set_bit(self.registers.l, bit_position),
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = set_bit(value, bit_position);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::SRL(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = shift_right_logical(self, self.registers.a)
                    }
                    PrefixTarget::B => {
                        self.registers.b = shift_right_logical(self, self.registers.b)
                    }
                    PrefixTarget::C => {
                        self.registers.c = shift_right_logical(self, self.registers.c)
                    }
                    PrefixTarget::D => {
                        self.registers.d = shift_right_logical(self, self.registers.d)
                    }
                    PrefixTarget::E => {
                        self.registers.e = shift_right_logical(self, self.registers.e)
                    }
                    PrefixTarget::H => {
                        self.registers.h = shift_right_logical(self, self.registers.h)
                    }
                    PrefixTarget::L => {
                        self.registers.l = shift_right_logical(self, self.registers.l)
                    }
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = shift_right_logical(self, value);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::RR(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = rotate_right_through_carry(self, self.registers.a, true)
                    }

                    PrefixTarget::B => {
                        self.registers.b = rotate_right_through_carry(self, self.registers.b, true)
                    }

                    PrefixTarget::C => {
                        self.registers.c = rotate_right_through_carry(self, self.registers.c, true)
                    }

                    PrefixTarget::D => {
                        self.registers.d = rotate_right_through_carry(self, self.registers.d, true)
                    }

                    PrefixTarget::E => {
                        self.registers.e = rotate_right_through_carry(self, self.registers.e, true)
                    }

                    PrefixTarget::H => {
                        self.registers.h = rotate_right_through_carry(self, self.registers.h, true)
                    }

                    PrefixTarget::L => {
                        self.registers.l = rotate_right_through_carry(self, self.registers.l, true)
                    }

                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = rotate_right_through_carry(self, value, true);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::RL(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = rotate_left_through_carry(self, self.registers.a, true)
                    }
                    PrefixTarget::B => {
                        self.registers.b = rotate_left_through_carry(self, self.registers.b, true)
                    }
                    PrefixTarget::C => {
                        self.registers.c = rotate_left_through_carry(self, self.registers.c, true)
                    }
                    PrefixTarget::D => {
                        self.registers.d = rotate_left_through_carry(self, self.registers.d, true)
                    }
                    PrefixTarget::E => {
                        self.registers.e = rotate_left_through_carry(self, self.registers.e, true)
                    }
                    PrefixTarget::H => {
                        self.registers.h = rotate_left_through_carry(self, self.registers.h, true)
                    }
                    PrefixTarget::L => {
                        self.registers.l = rotate_left_through_carry(self, self.registers.l, true)
                    }
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = rotate_left_through_carry(self, value, true);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::RRC(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = rotate_right(self, self.registers.a, true)
                    }
                    PrefixTarget::B => {
                        self.registers.b = rotate_right(self, self.registers.b, true)
                    }
                    PrefixTarget::C => {
                        self.registers.c = rotate_right(self, self.registers.c, true)
                    }
                    PrefixTarget::D => {
                        self.registers.d = rotate_right(self, self.registers.d, true)
                    }
                    PrefixTarget::E => {
                        self.registers.e = rotate_right(self, self.registers.e, true)
                    }
                    PrefixTarget::H => {
                        self.registers.h = rotate_right(self, self.registers.h, true)
                    }
                    PrefixTarget::L => {
                        self.registers.l = rotate_right(self, self.registers.l, true)
                    }
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = rotate_right(self, value, true);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::RLC(prefix) => {
                match prefix {
                    PrefixTarget::A => self.registers.a = rotate_left(self, self.registers.a, true),
                    PrefixTarget::B => self.registers.b = rotate_left(self, self.registers.b, true),
                    PrefixTarget::C => self.registers.c = rotate_left(self, self.registers.c, true),
                    PrefixTarget::D => self.registers.d = rotate_left(self, self.registers.d, true),
                    PrefixTarget::E => self.registers.e = rotate_left(self, self.registers.e, true),
                    PrefixTarget::H => self.registers.h = rotate_left(self, self.registers.h, true),
                    PrefixTarget::L => self.registers.l = rotate_left(self, self.registers.l, true),
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = rotate_left(self, value, true);
                        self.memory.write_byte(address, result);
                    }
                };

                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::SRA(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = shift_right_arithmetic(self, self.registers.a)
                    }
                    PrefixTarget::B => {
                        self.registers.b = shift_right_arithmetic(self, self.registers.b)
                    }
                    PrefixTarget::C => {
                        self.registers.c = shift_right_arithmetic(self, self.registers.c)
                    }
                    PrefixTarget::D => {
                        self.registers.d = shift_right_arithmetic(self, self.registers.d)
                    }
                    PrefixTarget::E => {
                        self.registers.e = shift_right_arithmetic(self, self.registers.e)
                    }
                    PrefixTarget::H => {
                        self.registers.h = shift_right_arithmetic(self, self.registers.h)
                    }
                    PrefixTarget::L => {
                        self.registers.l = shift_right_arithmetic(self, self.registers.l)
                    }
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = shift_right_arithmetic(self, value);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::SLA(prefix) => {
                match prefix {
                    PrefixTarget::A => {
                        self.registers.a = shift_left_arithmetic(self, self.registers.a)
                    }
                    PrefixTarget::B => {
                        self.registers.b = shift_left_arithmetic(self, self.registers.b)
                    }
                    PrefixTarget::C => {
                        self.registers.c = shift_left_arithmetic(self, self.registers.c)
                    }
                    PrefixTarget::D => {
                        self.registers.d = shift_left_arithmetic(self, self.registers.d)
                    }
                    PrefixTarget::E => {
                        self.registers.e = shift_left_arithmetic(self, self.registers.e)
                    }
                    PrefixTarget::H => {
                        self.registers.h = shift_left_arithmetic(self, self.registers.h)
                    }
                    PrefixTarget::L => {
                        self.registers.l = shift_left_arithmetic(self, self.registers.l)
                    }
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = shift_left_arithmetic(self, value);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }
            Instruction::SWAP(prefix) => {
                match prefix {
                    PrefixTarget::A => self.registers.a = swap_nibbles(self, self.registers.a),
                    PrefixTarget::B => self.registers.b = swap_nibbles(self, self.registers.b),
                    PrefixTarget::C => self.registers.c = swap_nibbles(self, self.registers.c),
                    PrefixTarget::D => self.registers.d = swap_nibbles(self, self.registers.d),
                    PrefixTarget::E => self.registers.e = swap_nibbles(self, self.registers.e),
                    PrefixTarget::H => self.registers.h = swap_nibbles(self, self.registers.h),
                    PrefixTarget::L => self.registers.l = swap_nibbles(self, self.registers.l),
                    PrefixTarget::HLI => {
                        let address = self.registers.hl();
                        let value = self.memory.read_byte(address);
                        let result = swap_nibbles(self, value);
                        self.memory.write_byte(address, result);
                    }
                };
                match prefix {
                    PrefixTarget::HLI => (self.registers.pc.wrapping_add(2), 16),
                    _ => (self.registers.pc.wrapping_add(2), 8),
                }
            }

            Instruction::RETI => {
                self.interrupts_enabled = true;
                (self.pop(), 16)
            }
            Instruction::RST(location) => {
                self.rst();
                (location.to_hex(), 24)
            }

            Instruction::HALT => {
                self.is_halted = true;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::NOP => (self.registers.pc.wrapping_add(1), 4),
            Instruction::DI => {
                self.interrupts_enabled = false;
                (self.registers.pc.wrapping_add(1), 4)
            }
            Instruction::EI => {
                self.interrupts_enabled = true;
                (self.registers.pc.wrapping_add(1), 4)
            } // _ => { /*add support for more instructions*/ }
        }
    }
}

fn alu_add(cpu: &mut Cpu, value: u8, with_carry: bool) -> u8 {
    let additional_carry = if with_carry && cpu.registers.flag_c {
        1
    } else {
        0
    };

    let (new_value, did_overflow) = cpu.registers.a.overflowing_add(value);
    let (new_value2, did_overflow2) = new_value.overflowing_add(additional_carry);
    cpu.registers.flag_z = new_value2 == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_c = did_overflow || did_overflow2;
    // TODO: check if the carry need's to be set to zero;

    // Half Carry is set if adding the lower nibbles of the value and register A
    // together result in a value bigger than 0xF. If the result is larger than 0xF
    // than the addition caused a carry from the lower nibble to the upper nibble.
    cpu.registers.flag_h = ((cpu.registers.a & 0xF) + (value & 0xF) + additional_carry) > 0xF;
    new_value2
}

fn alu_sub(cpu: &mut Cpu, value: u8, with_carry: bool) -> u8 {
    let additional_carry = if with_carry && cpu.registers.flag_c {
        1
    } else {
        0
    };

    let (new_value, did_overflow) = cpu.registers.a.overflowing_sub(value);
    let (new_value2, did_overflow2) = new_value.overflowing_sub(additional_carry);
    cpu.registers.flag_z = new_value2 == 0;
    cpu.registers.flag_n = true;
    cpu.registers.flag_c = did_overflow || did_overflow2;
    // TODO: check if the carry need's to be set to zero;

    // Half Carry is set if adding the lower nibbles of the value and register A
    // together result in a value bigger than 0xF. If the result is larger than 0xF
    // than the addition caused a carry from the lower nibble to the upper nibble.
    cpu.registers.flag_h = (cpu.registers.a & 0xF) < (value & 0xF) + additional_carry;
    new_value2
}

fn inc_8bit(cpu: &mut Cpu, value: u8) -> u8 {
    let new_value = value.wrapping_add(1);
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    // Half Carry is set if the lower nibble of the value is equal to 0xF.
    // If the nibble is equal to 0xF (0b1111) that means incrementing the value
    // by 1 would cause a carry from the lower nibble to the upper nibble.
    cpu.registers.flag_h = value & 0xF == 0xF;
    new_value
}

fn dec_8bit(cpu: &mut Cpu, value: u8) -> u8 {
    let new_value = value.wrapping_sub(1);
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = true;
    // Half Carry is set if the lower nibble of the value is equal to 0x0.
    // If the nibble is equal to 0x0 (0b0000) that means decrementing the value
    // by 1 would cause a carry from the upper nibble to the lower nibble.
    cpu.registers.flag_h = value & 0xF == 0x0;
    new_value
}

fn decimal_adjust(cpu: &mut Cpu, value: u8) -> u8 {
    let mut carry = false;

    let result = if !cpu.registers.flag_z {
        let mut result = value;
        if cpu.registers.flag_c || value > 0x99 {
            carry = true;
            result = result.wrapping_add(0x60);
        }
        if cpu.registers.flag_h || value & 0x0F > 0x09 {
            result = result.wrapping_add(0x06);
        }
        result
    } else if cpu.registers.flag_c {
        carry = true;
        let add = if cpu.registers.flag_h { 0x9A } else { 0xA0 };
        value.wrapping_add(add)
    } else if cpu.registers.flag_h {
        value.wrapping_add(0xFA)
    } else {
        value
    };

    cpu.registers.flag_z = result == 0;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = carry;

    result
}

fn add_hl(cpu: &mut Cpu, value: u16) -> u16 {
    let (new_value, did_overflow) = cpu.registers.hl().overflowing_add(value);
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_c = did_overflow;
    new_value
}

fn rotate_right_through_carry(cpu: &mut Cpu, value: u8, set_zero: bool) -> u8 {
    let carry_bit = if cpu.registers.flag_c { 1 } else { 0 } << 7;
    let new_value = carry_bit | (value >> 1);
    cpu.registers.flag_z = set_zero && new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = value & 0b1 == 0b1;
    new_value
}

fn rotate_left_through_carry(cpu: &mut Cpu, value: u8, set_zero: bool) -> u8 {
    let carry_bit = if cpu.registers.flag_c { 1 } else { 0 };
    let new_value = (value << 1) | carry_bit;
    cpu.registers.flag_z = set_zero && new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = (value & 0x80) == 0x80;
    new_value
}

fn rotate_left(cpu: &mut Cpu, value: u8, set_zero: bool) -> u8 {
    let carry = (value & 0x80) >> 7;
    let new_value = value.rotate_left(1) | carry;
    cpu.registers.flag_z = set_zero && new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = carry == 0x01;
    new_value
}

fn rotate_right(cpu: &mut Cpu, value: u8, set_zero: bool) -> u8 {
    let new_value = value.rotate_right(1);
    cpu.registers.flag_z = set_zero && new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = value & 0b1 == 0b1;
    new_value
}

fn bit_test(cpu: &mut Cpu, value: u8, bit_position: BitPosition) {
    let bit_position: u8 = bit_position.into();
    let result = (value >> bit_position) & 0b1;
    cpu.registers.flag_z = result == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = true;
}

fn reset_bit(value: u8, bit_position: BitPosition) -> u8 {
    let bit_position: u8 = bit_position.into();
    value & !(1 << bit_position)
}

fn set_bit(value: u8, bit_position: BitPosition) -> u8 {
    let bit_position: u8 = bit_position.into();
    value | (1 << bit_position)
}

fn shift_right_logical(cpu: &mut Cpu, value: u8) -> u8 {
    let new_value = value >> 1;
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = value & 0b1 == 0b1;
    new_value
}

fn shift_right_arithmetic(cpu: &mut Cpu, value: u8) -> u8 {
    let msb = value & 0x80;
    let new_value = msb | (value >> 1);
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = value & 0b1 == 0b1;
    new_value
}

fn shift_left_arithmetic(cpu: &mut Cpu, value: u8) -> u8 {
    let new_value = value << 1;
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = value & 0x80 == 0x80;
    new_value
}

fn swap_nibbles(cpu: &mut Cpu, value: u8) -> u8 {
    let new_value = ((value & 0xf) << 4) | ((value & 0xf0) >> 4);
    cpu.registers.flag_z = new_value == 0;
    cpu.registers.flag_n = false;
    cpu.registers.flag_h = false;
    cpu.registers.flag_c = false;
    new_value
}
