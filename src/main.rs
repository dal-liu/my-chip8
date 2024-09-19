extern crate sdl2;

use my_chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::Duration;

fn main() {
    let mut chip8 = Chip8::new();

    let rom_path = "roms/IBM Logo.ch8";
    chip8.load_rom(&rom_path);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(&rom_path[5..rom_path.len() - 4], 1280, 640)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_scale(20.0, 20.0).unwrap();
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        chip8.run();

        if chip8.draw_flag() {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();

            canvas.set_draw_color(Color::WHITE);
            chip8.display().iter().enumerate().for_each(|(i, &pixel)| {
                if pixel == 1 {
                    let x = (i % 64).try_into().unwrap();
                    let y = (i / 64).try_into().unwrap();
                    canvas.draw_point(Point::new(x, y)).unwrap();
                }
            });

            canvas.present();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_secs_f64(1.0 / 700.0));
    }
}
