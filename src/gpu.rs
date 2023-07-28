use std::collections::VecDeque;

use minifb::Window;
use minifb::WindowOptions;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

use crate::cpu::memory::Memory;

pub struct Gpu {
    window: Window,
}

impl Gpu {
    pub fn new() -> Gpu {

        // Initialize the window using minifb
        let window = Window::new(
            "Gameboy Emulator",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: false,
                scale: minifb::Scale::X4,
                ..WindowOptions::default()
            },
        ).unwrap();

        Gpu { window }
    }

    pub fn render(&mut self, memory: &Memory) {
        if self.window.is_open() {
            let tileset = memory.tileset();
            let tile_vec = tile_to_vec(&tileset[0+2..16+2]);
            // Testing tile: FF 00 7E FF 85 81 89 83 93 85 A5 8B C9 97 7E FF
            // let tileset: [u8; 16] = [0xFF, 0x00, 0x7E, 0xFF, 0x85, 0x81, 0x89, 0x83, 0x93, 0x85, 0xA5, 0x8B, 0xC9, 0x97, 0x7E, 0xFF];
            // let tile_vec: Vec<u8> = tile_to_vec(&tileset);
            let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

            for x in 0..8 {
                for y in 0..8 {
                    let color = tile_vec[x + y * 8];
                    // print_tile(&tile_vec);
                    let rgb = byte_to_rgb(color);
                    buffer[x + y * WIDTH] = rgb;
                }
            }

            self.window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    }

}
fn tile_to_vec(tile: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();

    for i in (0..8) {
        let i = i * 2;
        let byte1 = tile[i];
        let byte2 = tile[i + 1];

        for j in 0..8 {
            let bit1 = (byte1 & (1 << (7 - j))) >> (7 - j);
            let bit2 = (byte2 & (1 << (7 - j))) >> (7 - j);
            let color = bit1 | (bit2 << 1);
            vec.push(color);
        }
    }
    // println!("{:?}", vec);
    vec
}

fn print_tile(tile: &Vec<u8>) {
    for i in 0..8 {
        for j in 0..8 {
            let color = tile[i + j * 8];
            let rgb = byte_to_rgb(color);
            print!("| {:x} |\t", color);
        }
        println!("\n-----------------");
    }
    println!("........................");
}

fn byte_to_rgb(byte: u8) -> u32 {
    match byte {
        0 => 0x00FFFFFF,
        1 => 0x00AAAAAA,
        2 => 0x00555555,
        3 => 0x00000000,
        _ => panic!("Invalid byte"),
    }
}