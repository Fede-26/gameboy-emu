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

    pub memory: [u8; 0xFFFF+1],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; 0xFFFF+1],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0xFFFF => self.memory[address as usize],
            _ => panic!("Address out of range: 0x{:x}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0xFFFF => self.memory[address as usize] = value,

            _ => panic!("Address out of range: 0x{:x}", address),
        }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        self.memory[0..rom.len()].copy_from_slice(rom);
        // Copy the rom vector into the rom_bank_0 and rom_bank_1 arrays like a single contiguous array
        // self.rom_bank_0.copy_from_slice(&rom[0..0x4000]);
        // self.rom_bank_1.copy_from_slice(&rom[0x4000..0x8000]);
    }

    pub fn tileset(&self) -> &[u8] {
        &self.memory[0x8000..0x9800]
    }
}
