#[allow(unused_variables)]
#[allow(unused_imports)]
extern crate sdl2;

use std::arch::x86_64;
use std::num::Wrapping;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use array2d::{Array2D, Error};
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use std::thread::sleep;
use std::path::Path;
use rand::prelude::*;
use std::f32::consts;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::video::Window;

struct Chip8 {
    cpu_frequency: u32, // cpu speed
    _PC: u16, // program counter
    v_registers: [u8; 16], // cpu general purpose registers
    index_register: u16, // index register
    _delay_timer: u8, // delay timer (counts down at 60 HZ regardless of cpu speed)
    _sound_timer: u8, // sound timer (counts down at 60 HZ regardless of cpu speed, beep should play while sound timer is greater than 0)
    memory: [u8; 4096], // memory
    stack: Vec<u16>, // stack
    frame_buffer: Array2D<u8>, // frame buffer for display
    display_size_x: u16, // width of display
    display_size_y: u16, // height of display
    screen_scale: u16, // change scale of display
    input_array: [bool; 16], // flags for keys being pressed
    font_starting_memory_addr: u16
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            cpu_frequency: 500,
            _PC: 0x200, // Normal program start position at 0x200
            v_registers: [0; 16],
            index_register: 0,
            _delay_timer: 0,
            _sound_timer: 0,
            memory: [0; 4096],
            stack: Vec::new(),
            display_size_x: 64,
            display_size_y: 32,
            screen_scale: 20,
            // TODO: the values for display size x and y must be entered manually into frame_buffer
            // fix so display_size_x/y and frame_buffer get x and y values from same place
            frame_buffer: Array2D::filled_with(0, 64, 32),
            input_array: [false; 16],
            font_starting_memory_addr: 0x50
        }
    }
}

fn main() {
    // Chip8
    let mut chip8 = Chip8::default();

    // Game FPS
    let fps = 60;

    // Game file
    let file: String = String::from("C:\\Users\\jappa\\Repos\\chip8_rust\\src\\PONG.ch8");    

    // Read file into memory
    let mem_idx = chip8._PC;
    let buf = std::fs::read(file).unwrap();
    for (idx, byte) in buf.iter().enumerate() {
        chip8.memory[(mem_idx as usize) + idx] = *byte;
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
    let mut i:usize = chip8.font_starting_memory_addr as usize;
    for val in font {
        chip8.memory[i] = val;
        i += 1;
    }

    // Print time
    let mut curr_time = SystemTime::now();
    let delay_timer = 60;

    // sdl2 events and video
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Build window
    let window = video_subsystem.window("Chip-8", (chip8.display_size_x * chip8.screen_scale) as u32, (chip8.display_size_y * chip8.screen_scale) as u32)
    .position_centered()
    .build()
    .unwrap();

    // Canvas
    let mut canvas: Canvas<Window> = window.into_canvas().present_vsync().build().unwrap();


    // Game loop
    'game_loop: loop {
        let op_code = fetch(&mut chip8);
        decode(op_code, &mut chip8, &mut canvas);

        if curr_time.elapsed().unwrap().as_millis() > delay_timer {
            curr_time = SystemTime::now();
            if chip8._delay_timer > 0 {chip8._delay_timer -= 1};
            if chip8._sound_timer > 0 {chip8._sound_timer -= 1};
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Num1), ..} => {
                    chip8.input_array[0x1] = true;
                    println!("1 down");
                }

                Event::KeyUp {keycode: Some(Keycode::Num1), ..} => {
                    chip8.input_array[0x1] = false;
                    println!("1 up");
                }

                Event::KeyDown {keycode: Some(Keycode::Num2), ..} => {
                    chip8.input_array[0x2] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::Num2), ..} => {
                    chip8.input_array[0x2] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::Num3), ..} => {
                    chip8.input_array[0x3] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::Num3), ..} => {
                    chip8.input_array[0x3] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::Num4), ..} => {
                    chip8.input_array[0xC] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::Num4), ..} => {
                    chip8.input_array[0xC] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::Q), ..} => {
                    chip8.input_array[0x4] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::Q), ..} => {
                    chip8.input_array[0x4] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::W), ..} => {
                    chip8.input_array[0x5] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::W), ..} => {
                    chip8.input_array[0x5] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::E), ..} => {
                    chip8.input_array[0x6] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::E), ..} => {
                    chip8.input_array[0x6] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::R), ..} => {
                    chip8.input_array[0xD] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::R), ..} => {
                    chip8.input_array[0xD] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::A), ..} => {
                    chip8.input_array[0x7] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::A), ..} => {
                    chip8.input_array[0x7] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::S), ..} => {
                    chip8.input_array[0x8] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::S), ..} => {
                    chip8.input_array[0x8] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::D), ..} => {
                    chip8.input_array[0x9] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::D), ..} => {
                    chip8.input_array[0x9] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
                    chip8.input_array[0xA] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::Z), ..} => {
                    chip8.input_array[0xA] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::X), ..} => {
                    chip8.input_array[0x0] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::X), ..} => {
                    chip8.input_array[0x0] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::C), ..} => {
                    chip8.input_array[0xB] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::C), ..} => {
                    chip8.input_array[0xB] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::V), ..} => {
                    chip8.input_array[0xF] = true;
                }

                Event::KeyUp {keycode: Some(Keycode::V), ..} => {
                    chip8.input_array[0xF] = false;
                }

                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'game_loop;
                }

                _ => {}
            }

        sleep(Duration::new(0, 1_000_000_000 / chip8.cpu_frequency));
        }
    }
}

fn fetch(chip8: &mut Chip8) -> u16{
    let instruct1: u16 = u16::from(chip8.memory[chip8._PC as usize]) << 8;
    let instruct2: u16 = u16::from(chip8.memory[(chip8._PC + 1) as usize]);
    let full_instruct: u16 = instruct1 + instruct2;
    // println!("{:x}", full_instruct);
    chip8._PC += 2;
    full_instruct
}

fn decode(op_code: u16, chip8: &mut Chip8, canvas: &mut Canvas<Window>) {
    let first_nibble = (op_code & 0b1111000000000000) >> 12;
    let second_nibble = (op_code & 0b0000111100000000) >> 8;
    let third_nibble = (op_code & 0b0000000011110000) >> 4;
    let fourth_nibble = op_code & 0b0000000000001111;

    let op_x = &second_nibble;
    let op_y = &third_nibble;
    let op_n = &fourth_nibble;
    let op_nn: u8 = (op_code & 0b0000_0000_1111_1111).try_into().unwrap();
    let op_nnn = (op_code & 0b0000_1111_1111_1111);

    // println!("1. {:x} 2. {:x} 3. {:x} 4. {:x}", first_nibble, second_nibble, third_nibble, fourth_nibble);

    if op_code == 0x00E0 {
        // println!("Clear Screen");
        clear_screen(canvas);
    }
    if op_code == 0x00EE {
        chip8._PC = chip8.stack.pop().unwrap();
    }

    match first_nibble {
        0x1 => {
            // println!("Jump");
            chip8._PC = op_nnn;
        }

        0x2 => {
            // println!("Subroutine")
            chip8.stack.push(chip8._PC);
            chip8._PC = op_nnn;
        }

        0x3 => {
            // println!("Skip if VX == NN")
            if (chip8.v_registers[*op_x as usize] == op_nn) {
                chip8._PC += 2;
            }
        }

        0x4 => {
            // println!("Skip if VX != NN")
            if (chip8.v_registers[*op_x as usize] != op_nn) {
                chip8._PC += 2;
            }
        }

        0x5 => {
            // println!("Skip if VX == VY")
            if (chip8.v_registers[*op_x as usize] == chip8.v_registers[*op_y as usize]) {
                chip8._PC += 2;
            }
        }

        0x6 => {
            // println!("Set register VX to {}", op_nn);
            chip8.v_registers[*op_x as usize] = op_nn;
        }

        0x7 => {
            // println!("Add {} register VX", op_nn);
            let new_val = chip8.v_registers[*op_x as usize] as u16 + op_nn as u16;
            chip8.v_registers[*op_x as usize] = (new_val & 0b1111_1111) as u8;
        }

        0x8 => {
            match fourth_nibble {
                0x0 => {
                    // Set VX = VY
                    chip8.v_registers[*op_x as usize] = chip8.v_registers[*op_y as usize];
                }

                0x1 => {
                    // VX |= VY
                    chip8.v_registers[*op_x as usize] |= chip8.v_registers[*op_y as usize];
                }

                0x2 => {
                    // VX &= VY
                    chip8.v_registers[*op_x as usize] &= chip8.v_registers[*op_y as usize];
                }

                0x3 => {
                    // VX ^= VY
                    chip8.v_registers[*op_x as usize] ^= chip8.v_registers[*op_y as usize];
                }

                0x4 => {
                    // VX += VY
                    let new_val = chip8.v_registers[*op_x as usize] as u16 + chip8.v_registers[*op_y as usize] as u16;
                    if  new_val > 255 {
                        chip8.v_registers[0xF] = 1;
                    }
                    chip8.v_registers[*op_x as usize] = (new_val & 0b1111_1111) as u8;
                }

                0x5 => {
                    // VX -= VY
                    if chip8.v_registers[*op_x as usize] > chip8.v_registers[*op_y as usize] {
                        chip8.v_registers[0xF] = 1;
                        chip8.v_registers[*op_x as usize] -= chip8.v_registers[*op_y as usize];
                    }
                    else {
                        chip8.v_registers[0xF] = 0;
                        let x_val = Wrapping(chip8.v_registers[*op_x as usize]);
                        let y_val = Wrapping(chip8.v_registers[*op_y as usize]);
                        let new_val = x_val - y_val;
                        chip8.v_registers[*op_x as usize] = new_val.0;
                    }
                }

                0x6 => {
                    // VX shift right
                    // chip8.v_registers[*op_x as usize] = chip8.v_registers[*op_y as usize]; // TODO: Make optional

                    if chip8.v_registers[*op_x as usize] & 0x1 == 1 {
                        println!("{:b}", chip8.v_registers[*op_x as usize]);
                        chip8.v_registers[0xF] = 1;
                    }
                    else {
                        chip8.v_registers[0xF] = 0;
                    }

                    chip8.v_registers[*op_x as usize] >>= 1;
                    println!("{:b}", chip8.v_registers[*op_x as usize]);
                    println!("VF {}", chip8.v_registers[0xF]);
                }                

                0x7 => {
                    // VX = VY - VX
                    if chip8.v_registers[*op_y as usize] > chip8.v_registers[*op_x as usize] {
                        chip8.v_registers[0xF] = 1;
                        chip8.v_registers[*op_x as usize] = chip8.v_registers[*op_y as usize] - chip8.v_registers[*op_x as usize];
                    }
                    else {
                        chip8.v_registers[0xF] = 0;
                        let x_val = Wrapping(chip8.v_registers[*op_x as usize]);
                        let y_val = Wrapping(chip8.v_registers[*op_y as usize]);
                        let new_val = y_val - x_val;
                        chip8.v_registers[*op_x as usize] = new_val.0;
                    }

                }

                0xE => {
                    // VX shift left
                    // chip8.v_registers[*op_x as usize] = chip8.v_registers[*op_y as usize]; // TODO: Make optional

                    if chip8.memory[*op_x as usize] >> 7 & 0x1 == 1 {
                        chip8.memory[0xF] = 1;
                    }
                    else {
                        chip8.memory[0xF] = 0;
                    }
                    
                    chip8.memory[*op_x as usize] <<= 1;
                } 

                _ => {
                    println!("Error: Cannot interpret 8XXX opcode");
                }   
            }
        }

        0x9 => {
            // println!("Skip if VX != VY")
            if (chip8.v_registers[*op_x as usize] != chip8.v_registers[*op_y as usize]) {
                chip8._PC += 2;
            }
        }

        0xA => {
            // Set index
            chip8.index_register = op_nnn;
        }

        0xB => {
            // Jump with offset
            // BNNN originally jumped to NNN + V0 but was changed in CHIP-48 and SUPER-CHIP to BXNN where it jumped to XNN + VX (i.e. B220 jumps to 220 plus V2)
            // Original method is used here
            chip8._PC = op_nnn + chip8.v_registers[0x0] as u16;
        }

        0xC => {
            // Random number & NN into VX
            let random_num = rand::random::<u8>() & op_nn;
            chip8.v_registers[*op_x as usize] = random_num;
        }

        0xD => {
            let mut x_coord = chip8.v_registers[*op_x as usize] % chip8.display_size_x as u8;
            let mut y_coord = chip8.v_registers[*op_y as usize] % chip8.display_size_y as u8;

            chip8.v_registers[0xF] = 0;

            for n in 0..*op_n {
                let sprite_data = chip8.memory[(chip8.index_register + n) as usize];
                for i in (0..8).rev() {
                    let bit = (sprite_data >> i) & 1;
                    if bit == 1 {
                        if chip8.frame_buffer[(x_coord as usize, y_coord as usize)] == 1 {
                            chip8.frame_buffer[(x_coord as usize, y_coord as usize)] = 0;
                            chip8.v_registers[0xF] = 1;
                        }
                        else {
                            chip8.frame_buffer[(x_coord as usize, y_coord as usize)] = 1;
                        }
                    }
                    else {
                        chip8.frame_buffer[(x_coord as usize, y_coord as usize)] = 0;
                    }
                    x_coord += 1;
                    if x_coord >= chip8.display_size_x as u8 {
                        break;
                    }
                }
                y_coord += 1;
                if y_coord >= chip8.display_size_y as u8 {
                    break;
                }
                x_coord = chip8.v_registers[*op_x as usize] % 64;
            }
            draw(&chip8, canvas);
        }

        0xE => {
            match fourth_nibble {
                0xE => {
                    // Skip one instruction if key X is being pressed
                    if chip8.input_array[chip8.memory[*op_x as usize] as usize] {
                        chip8._PC += 2;
                    }
                }

                0x1 => {
                    // Skip one instruction if key X is not being pressed
                    if !chip8.input_array[chip8.memory[*op_x as usize] as usize]  {
                        chip8._PC += 2;
                    }
                }

                _ => {
                    println!("Error: Cannot interpret 0xEXXX opcode");
                }
            }
        }

        0xF => {
            match op_nn {
                // Timers
                0x07 => {
                    // Set VX to current value of delay timer
                    chip8.v_registers[*op_x as usize] = chip8._delay_timer;
                }

                0x15 => {
                    // Set delay timer to VX
                    chip8._delay_timer = chip8.v_registers[*op_x as usize];
                }

                0x18 => {
                    // Set sound timer to VX
                    chip8._sound_timer = chip8.v_registers[*op_x as usize];
                }

                // Add to index
                0x1E => {
                    // Add VX to index register
                    chip8.index_register += chip8.v_registers[*op_x as usize] as u16;
                }

                // Get key
                0x0A => {
                    // Get key (blocks execution until key is pressed)
                    // Delay and sound timers should still increment
                    let mut key_pressed = false;
                    for (i, key) in chip8.input_array.iter().enumerate() {
                        if *key == true {
                            chip8.memory[*op_x as usize] = i as u8;
                            key_pressed = true;
                        }
                    }

                    if !key_pressed {
                        chip8._PC -= 2;
                    }
                }

                // Font character
                0x29 => {
                    // Sets index register to address of hexadecimal character in VX
                    match chip8.memory[*op_x as usize] {
                        0x0 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0);
                        }

                        0x1 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 1);
                        }

                        0x2 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 2);
                        }

                        0x3 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 3);
                        }

                        0x4 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 4);
                        }

                        0x5 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 5);
                        }

                        0x6 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 6);
                        }

                        0x7 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 7);
                        }

                        0x8 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 8);
                        }

                        0x9 => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 9);
                        }

                        0xA => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xA);
                        }

                        0xB => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xB);
                        }

                        0xC => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xC);
                        }

                        0xD => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xD);
                        }

                        0xE => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xE);
                        }

                        0xF => {
                            chip8.index_register = chip8.font_starting_memory_addr + (5 * 0xF);
                        }

                        _ => {
                            println!("Error: Invalid key input");
                        }
                    }
                }

                // Binary coded decimal division
                // TODO: Improve this!
                0x33 => {
                    // Converts number in VX to three decimal digits and stores in addresses at I, I+1, and I+2 (i.e. 156 would be .156 and stored as mem[I] = 1, mem[I+1] = 5, mem[I+2] = 6)
                    let number = chip8.v_registers[*op_x as usize];
                    let first_digit = (number as f32 / 100f32).floor();
                    let second_digit =  ((number as f32 - first_digit * 100f32) / 10f32).floor();
                    let third_digit = number as f32 - (first_digit * 100f32) - (second_digit * 10f32);

                    chip8.memory[chip8.index_register as usize] = first_digit as u8;
                    chip8.memory[(chip8.index_register + 1) as usize] = second_digit as u8;
                    chip8.memory[(chip8.index_register + 2) as usize] = third_digit as u8;
                }

                // Store and load memory
                // The original CHIP-8 interpreter increments index register (I) but newer versions leave I at original value
                // I is left alone (like newer CHIP-8 versions)
                // TODO: Add older CHIP-8 interpreter functionality option
                0x55 => {
                    // Stores value VX(+i) in mem[i+1]
                    for i in 0..*op_x + 1 {
                        chip8.memory[(chip8.index_register + i) as usize] = chip8.v_registers[i as usize];
                    }
                }

                0x65 => {
                    // Stores value mem[i+1] in VX(+i)
                    for i in 0..*op_x + 1 {
                        chip8.v_registers[i as usize] = chip8.memory[(chip8.index_register + i) as usize];
                    }
                }

                _ => {
                    println!("Error: Cannot interpret FXXX opcode");
                }
            }
        }

        _ => {}
    } 

}


// Draw to screen
fn draw(chip8: &Chip8, canvas: &mut Canvas<Window>) {
    for y in 0..32 {
        for x in 0..64 {
            if chip8.frame_buffer[(x as usize, y as usize)] == 1 {
                canvas.set_draw_color(Color::RGB(0, 0, 255));
                let pt = Point::new(x * chip8.screen_scale as i32, y * chip8.screen_scale as i32);
                let big_pt = Rect::from_center(pt, chip8.screen_scale as u32, chip8.screen_scale as u32);
                canvas.fill_rect(big_pt);
                canvas.draw_rect(big_pt);
            }
            else {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                let pt = Point::new(x * chip8.screen_scale as i32, y * chip8.screen_scale as i32);
                let big_pt = Rect::from_center(pt, 10, 10);
                canvas.draw_rect(big_pt);
            }
        }
    }
    canvas.present();
}

// TODO: Need to clear frame_buffer array as well
fn clear_screen(canvas: &mut Canvas<Window>) {
    canvas.clear();
}