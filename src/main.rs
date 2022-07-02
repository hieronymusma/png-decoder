use std::{error::Error, time::Duration};

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
    EventPump,
};

type Res = Result<(), Box<dyn Error>>;

fn main() -> Res {
    let file_argument = std::env::args().nth(1).expect("No png file path is given");
    let file_path = std::path::PathBuf::from(file_argument);
    let png_data = std::fs::read(file_path)?;

    process(&png_data)?;

    Ok(())
}

const MAGIC_NUMBER: &[u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn process(png: &[u8]) -> Res {
    assert_eq!(
        MAGIC_NUMBER,
        &png[..MAGIC_NUMBER.len()],
        "Magic number does not match.\nExpected:\n{:02x?}\nGot:\n{:02x?}",
        MAGIC_NUMBER,
        &png[..MAGIC_NUMBER.len()]
    );

    let (mut canvas, event_pump) = create_canvas(800, 600)?;

    for h in 0..600 {
        for w in 0..800 {
            let c = match h % 3 {
                0 => Color::RGB(255, 0, 0),
                1 => Color::RGB(0, 255, 0),
                2 => Color::RGB(0, 0, 255),
                _ => panic!(),
            };
            canvas.set_draw_color(c);
            canvas.draw_point(Point::new(w, h))?;
        }
    }

    canvas.present();

    run_eventloop(event_pump);

    Ok(())
}

fn run_eventloop(mut event_pump: EventPump) {
    'running: loop {
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
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}

fn create_canvas(width: u32, height: u32) -> Result<(Canvas<Window>, EventPump), Box<dyn Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", width, height)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let event_pump = sdl_context.event_pump()?;

    Ok((canvas, event_pump))
}
