use core::prelude;
#[allow(unused_variables)]
#[allow(unused_imports)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use array2d::{Array2D, Error};
use device_query::{DeviceQuery, DeviceState, Keycode};

fn main() {
    // CPU speed
    let cpu_frequency: u64 = 500;

    // Game file
    let file: String = String::from("C:\\Users\\jappa\\Repos\\chip8_rust\\src\\IBM_Logo.ch8");    

    // Program counter (Programs usually start at 0x200)
    let mut _PC:u32 = 0x200;

    // Memory
    let mut memory: [u8; 4096] = [0; 4096];

    // Read file into memory
    let mem_idx = &_PC;
    let buf = std::fs::read(file).unwrap();
    for (idx, byte) in buf.iter().enumerate() {
        memory[(*mem_idx as usize) + idx] = *byte;
    }

    // Font
    let font: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    // Add font to memory
    let mut i:usize = 0x50;
    for val in font {
        memory[i] = val;
        i += 1;
    }

    // Frame buffer
    let mut frame_buffer: Array2D<u8> = Array2D::filled_with(0, 64, 32);

    // CPU registers
    let mut v_registers: [u8; 16] = [0; 16];

    // Index register
    let mut index_register: u16 = 0;

    // Stack
    let mut stack: Vec<u16> = Vec::new();

    // Timers
    let mut _delay_timer: u8 = 0;
    let mut _sound_timer: u8 = 0;

    let device_state = DeviceState::new();
    // Print time
    let mut currTime = SystemTime::now();

    // Game loop
    while true {
        if (currTime.elapsed().unwrap() > Duration::from_millis(cpu_frequency)) {
            currTime = SystemTime::now();
        
            let op_code = fetch(&mut _PC, &memory);
            decode(op_code, &mut _PC, &mut stack, &mut memory, &mut frame_buffer, &mut v_registers, &mut index_register);
            // println!("{}", _PC);

            let keys: Vec<Keycode> = device_state.get_keys();
            if keys.contains(&Keycode::Key1) {
                println!("1");
            }
            if keys.contains(&Keycode::Key2) {
                println!("2");
            }
            if keys.contains(&Keycode::Key3) {
                println!("3");
            }
            if keys.contains(&Keycode::Key4) {
                println!("C");
            }
            if keys.contains(&Keycode::Q) {
                println!("4");
            }
            if keys.contains(&Keycode::W) {
                println!("5");
            }
            if keys.contains(&Keycode::E) {
                println!("6");
            }
            if keys.contains(&Keycode::R) {
                println!("D");
            }
            if keys.contains(&Keycode::A) {
                println!("7");
            }
            if keys.contains(&Keycode::S) {
                println!("8");
            }
            if keys.contains(&Keycode::D) {
                println!("9");
            }
            if keys.contains(&Keycode::F) {
                println!("E");
            }
            if keys.contains(&Keycode::Z) {
                println!("A");
            }
            if keys.contains(&Keycode::X) {
                println!("0");
            }
            if keys.contains(&Keycode::C) {
                println!("B");
            }
            if keys.contains(&Keycode::V) {
                println!("F");
            }
        }
    }

}

fn fetch(_PC: &mut u32, memory: &[u8; 4096]) -> u16{
    let instruct1: u16 = u16::from(memory[*_PC as usize]) << 8;
    let instruct2: u16 = u16::from(memory[(*_PC + 1) as usize]);
    let full_instruct: u16 = instruct1 + instruct2;
    // println!("{:x}", full_instruct);
    *_PC += 2;
    full_instruct
}

fn decode(op_code: u16, _PC: &mut u32, _stack: &mut Vec<u16>, memory: &mut [u8; 4096], frame_buffer: &mut Array2D<u8>, v_registers: &mut [u8; 16], index_register: &mut u16) {
    let first_nibble = (op_code & 0b1111000000000000) >> 12;
    let second_nibble = (op_code & 0b0000111100000000) >> 8;
    let third_nibble = (op_code & 0b0000000011110000) >> 4;
    let fourth_nibble = op_code & 0b0000000000001111;

    let op_x = &second_nibble;
    let op_y = &third_nibble;
    let op_n = &fourth_nibble;
    let op_nn: u8 = (op_code & 0b0000000011111111).try_into().unwrap();
    let op_nnn = op_code & 0b0000111111111111;

    // println!("1. {:x} 2. {:x} 3. {:x} 4. {:x}", first_nibble, second_nibble, third_nibble, fourth_nibble);

    if op_code == 0x00E0 {
        // println!("Clear Screen");
        //clear_screen();
    }

    match first_nibble {
        0x1 => {
            // println!("Jump");
            *_PC = u32::from(op_nnn);
        }

        0x6 => {
            // println!("Set register VX");
            v_registers[*op_x as usize] = op_nn;
        }

        0x7 => {
            // println!("Set register VY");
            v_registers[*op_y as usize] = op_nn;
        }

        0xA => {
            // println!("Set index register");
            *index_register = op_nnn;
        }

        0xD => {
            println!("Draw");
            let mut x_coord = v_registers[*op_x as usize] % 64;
            let mut y_coord = v_registers[*op_y as usize] % 32;
            v_registers[0xF] = 0;

            for n in 0..*op_n {
                let sprite_data = memory[(*index_register + n) as usize];
                for i in (0..8).rev() {
                    let bit = &sprite_data >> i & 1;
                    if bit == 1 {
                        if frame_buffer[(x_coord as usize, y_coord as usize)] == 1 {
                            frame_buffer[(x_coord as usize, y_coord as usize)] = 0;
                            v_registers[0xF] = 1;
                        }
                        else {
                            frame_buffer[(x_coord as usize, y_coord as usize)] = 1;
                        }
                    }
                    else {
                        frame_buffer[(x_coord as usize, y_coord as usize)] = 0;
                    }
                   
                    x_coord += 1;
                }
                y_coord += 1;
                x_coord = v_registers[*op_x as usize] % 64;
            }
            draw(frame_buffer);
        }

        _ => {
            
        }
    } 

}

fn draw(frame_buffer: &Array2D<u8>) {
    for y in 0..32 {
        println!();
        for x in 0..64 {
            if frame_buffer[(x as usize, y as usize)] == 1 {
                print!("#");
            }
            else {
                print!(" ");
            }
        }
    }
    println!();
}