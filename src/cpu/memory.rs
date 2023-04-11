pub struct Memory {
    pub memory: [u8; 0xFFFF],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; 0xFFFF],
        }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        self.memory[0..rom.len()].copy_from_slice(rom);
    }
}
