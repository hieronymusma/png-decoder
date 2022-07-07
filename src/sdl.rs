use std::{error::Error, time::Duration};

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
    EventPump,
};

pub struct SDLWindow {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl SDLWindow {
    pub fn new(width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("png-decoder", width, height)
            .position_centered()
            .opengl()
            .build()?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump()?;

        Ok(Self { canvas, event_pump })
    }

    pub fn wait_for_close(&mut self) {
        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }
    }

    pub fn update(&mut self) {
        self.canvas.present();
    }

    pub fn draw_point(&mut self, color: Color, point: Point) -> Result<(), String> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_point(point)
    }
}
