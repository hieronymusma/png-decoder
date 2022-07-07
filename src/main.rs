use std::error::Error;

use sdl2::{pixels::Color, rect::Point};

mod crc;
mod png;
mod sdl;

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let file_argument = std::env::args().nth(1).expect("No png file path is given");
    let file_path = std::path::PathBuf::from(file_argument);
    let png_data = std::fs::read(file_path)?;

    process(&png_data)?;

    Ok(())
}

fn process(png: &[u8]) -> Res<()> {
    png::process(png)?;

    let mut canvas = sdl::SDLWindow::new(800, 600)?;

    for h in 0..600 {
        for w in 0..800 {
            let c = match h % 12 {
                0..=3 => Color::RGB(255, 0, 0),
                4..=7 => Color::RGB(0, 255, 0),
                8..=11 => Color::RGB(0, 0, 255),
                _ => panic!(),
            };
            canvas.draw_point(c, Point::new(w, h))?;
        }
    }

    canvas.update();
    canvas.wait_for_close();

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::crc;

    #[test]
    fn crc_test() {
        let mut crc = crc::CrcCalculator::new();
        crc.update(&*b"IEND");
        let result = crc.get_crc();
        println!("Result: {}", result);
    }
}
