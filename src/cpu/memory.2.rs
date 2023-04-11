pub struct Memory {
    // GB memory layout
    // 0x0000 - 0x3FFF: ROM bank 0
    // 0x4000 - 0x7FFF: ROM bank 1
    // 0x8000 - 0x9FFF: VRAM
    // 0xA000 - 0xBFFF: External RAM
    // 0xC000 - 0xCFFF: Work RAM bank 0
    // 0xD000 - 0xDFFF: Work RAM bank 1
    // 0xE000 - 0xFDFF: Echo RAM
    // 0xFE00 - 0xFE9F: Sprite Attribute Table (OAM)
    // 0xFEA0 - 0xFEFF: Not Usable
    // 0xFF00 - 0xFF7F: I/O Ports
    // 0xFF80 - 0xFFFE: High RAM (HRAM)
    // 0xFFFF: Interrupt Enable Register

    rom_bank_0: [u8; 0x4000],
    rom_bank_1: [u8; 0x4000],
    vram: [u8; 0x2000],
    external_ram: [u8; 0x2000],
    work_ram_bank_0: [u8; 0x1000],
    work_ram_bank_1: [u8; 0x1000],
    echo_ram: [u8; 0x1E00],
    sprite_attribute_table: [u8; 0x00A0],
    not_usable: [u8; 0x0060],
    io_ports: [u8; 0x0080],
    high_ram: [u8; 0x007F],
    interrupt_enable_register: u8,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            rom_bank_0: [0; 0x4000],
            rom_bank_1: [0; 0x4000],
            vram: [0; 0x2000],
            external_ram: [0; 0x2000],
            work_ram_bank_0: [0; 0x1000],
            work_ram_bank_1: [0; 0x1000],
            echo_ram: [0; 0x1E00],
            sprite_attribute_table: [0; 0x00A0],
            not_usable: [0; 0x0060],
            io_ports: [0; 0x0080],
            high_ram: [0; 0x007F],
            interrupt_enable_register: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_bank_0[address as usize],
            0x4000..=0x7FFF => self.rom_bank_1[address as usize],
            0x8000..=0x9FFF => self.vram[address as usize],
            0xA000..=0xBFFF => self.external_ram[address as usize],
            0xC000..=0xCFFF => self.work_ram_bank_0[address as usize],
            0xD000..=0xDFFF => self.work_ram_bank_1[address as usize],
            0xE000..=0xFDFF => self.echo_ram[address as usize],
            0xFE00..=0xFE9F => self.sprite_attribute_table[address as usize],
            0xFEA0..=0xFEFF => self.not_usable[address as usize],
            0xFF00..=0xFF7F => self.io_ports[address as usize],
            0xFF80..=0xFFFE => self.high_ram[address as usize],
            0xFFFF => self.interrupt_enable_register,
            _ => panic!("Address out of range: 0x{:x}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => self.rom_bank_0[address as usize] = value,
            0x4000..=0x7FFF => self.rom_bank_1[address as usize] = value,
            0x8000..=0x9FFF => self.vram[address as usize] = value,
            0xA000..=0xBFFF => self.external_ram[address as usize] = value,
            0xC000..=0xCFFF => self.work_ram_bank_0[address as usize] = value,
            0xD000..=0xDFFF => self.work_ram_bank_1[address as usize] = value,
            0xE000..=0xFDFF => self.echo_ram[address as usize] = value,
            0xFE00..=0xFE9F => self.sprite_attribute_table[address as usize] = value,
            0xFEA0..=0xFEFF => self.not_usable[address as usize] = value,
            0xFF00..=0xFF7F => self.io_ports[address as usize] = value,
            0xFF80..=0xFFFE => self.high_ram[address as usize] = value,
            0xFFFF => self.interrupt_enable_register = value,
            _ => panic!("Address out of range: 0x{:x}", address),
        }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        // self.memory[0..rom.len()].copy_from_slice(rom);
        // Copy the rom vector into the rom_bank_0 and rom_bank_1 arrays like a single contiguous array
        self.rom_bank_0.copy_from_slice(&rom[0..0x4000]);
        self.rom_bank_1.copy_from_slice(&rom[0x4000..0x8000]);
    }
}
