extern crate sdl2;

use my_chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

const BACKGROUND_COLOR: Color = Color::BLACK;
const CYCLES_PER_SECOND: f64 = 700.0;
const FOREGROUND_COLOR: Color = Color::WHITE;
const PIXEL_SIZE: f32 = 20.0;

fn main() {
    let mut chip8 = Chip8::new();

    let mut args = env::args().into_iter();
    args.next();
    match args.next() {
        Some(path) => chip8.load_rom(&path),
        None => panic!("Usage: cargo run <path-to-rom>"),
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", 1280, 640)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_scale(PIXEL_SIZE, PIXEL_SIZE).unwrap();
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    canvas.present();

    let scancode_to_key = HashMap::from([
        (Scancode::Num1, 0x0),
        (Scancode::Num2, 0x1),
        (Scancode::Num3, 0x2),
        (Scancode::Num4, 0x3),
        (Scancode::Q, 0x4),
        (Scancode::W, 0x5),
        (Scancode::E, 0x6),
        (Scancode::R, 0x7),
        (Scancode::A, 0x8),
        (Scancode::S, 0x9),
        (Scancode::D, 0xa),
        (Scancode::F, 0xb),
        (Scancode::Z, 0xc),
        (Scancode::X, 0xd),
        (Scancode::C, 0xe),
        (Scancode::V, 0xf),
    ]);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        chip8.run_cycle();

        if chip8.draw_flag() {
            canvas.set_draw_color(BACKGROUND_COLOR);
            canvas.clear();

            canvas.set_draw_color(FOREGROUND_COLOR);
            chip8.display().iter().enumerate().for_each(|(i, &pixel)| {
                if pixel == 1 {
                    let x = (i % my_chip8::DISPLAY_WIDTH) as i32;
                    let y = (i / my_chip8::DISPLAY_WIDTH) as i32;
                    canvas.draw_point(Point::new(x, y)).unwrap();
                }
            });

            canvas.present();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(&key) = scancode_to_key.get(&scancode) {
                        chip8.key_down(key);
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(&key) = scancode_to_key.get(&scancode) {
                        chip8.key_up(key);
                    }
                }
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_secs_f64(1.0 / CYCLES_PER_SECOND));
    }
}
