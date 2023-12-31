use std::env;
use std::fs;

mod rom_reader;
use rom_reader::{CartridgeType, RomHeader};

mod cpu;
use cpu::Cpu;

mod gpu;
use gpu::Gpu;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = &args[1];

    println!("Opening file {}", file_path);

    let rom_vec = fs::read(file_path).expect("Should have been able to read the file");

    // To test if the file is read correctly print the lenght of the file in Kbit hex
    println!(
        "File size: {} Kbit / 0x{:x} byte [0x0000 -> 0x{:x}] (N.{} Memory Banks)",
        rom_vec.len() * 8,
        rom_vec.len(),
        rom_vec.len() - 1,
        rom_vec.len() / 0x4000
    );

    // Create a new RomHeader struct from the vector
    let rom_header = RomHeader::from_vec(&rom_vec);

    // Print the title of the ROM, the ROM size and RAM size
    println!(
        "Title: {}",
        String::from_utf8(rom_header.title.to_vec()).unwrap()
    );
    println!("ROM size: {}", rom_header.rom_size);
    println!("RAM size: {}", rom_header.ram_size);
    // Print the cartridge type from the enum
    println!(
        "Cartridge type: {:?}",
        CartridgeType::from_u8(rom_header.cartridge_type).unwrap()
    );

    // Initialize the CPU
    let mut cpu = Cpu::new();

    cpu.memory.load_rom(&rom_vec);

    let mut gpu = Gpu::new();

    // Run the CPU
    let mut counter: u64 = 0;
    loop {
        // println!("Counter: {} | PC: {:#x}", counter, cpu.registers.pc);
        cpu.step();
        if counter%1000 == 0 {
            println!("Counter: {} | PC: {:#x}", counter, cpu.registers.pc);
        }
        // Print the registers
        // println!("Registers: {:#?}", cpu.registers);

        // Renders the screen
        gpu.render(&cpu.memory);

        // Sleep for a bit to slow down the execution
        // std::thread::sleep(std::time::Duration::from_micros(1));
        counter = counter.wrapping_add(1 as u64);

        // if counter > 5 {
        //     //wait
        //     loop {}
        // }
    }
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_header_title() {
        let rom_vec = fs::read("roms/tetris.gb").expect("Should have been able to read the file tetris.gb");

        let rom_header = RomHeader::from_vec(&rom_vec);
        
        assert_eq!(
            String::from_utf8(rom_header.title.to_vec()).unwrap().trim_end_matches(char::from(0)),
            "TETRIS"
        );
    }
}