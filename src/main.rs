use std::error::Error;

use sdl2::rect::Point;

mod crc;
mod filter;
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
    let png = png::process(png)?;
    let width = png.get_width();
    let height = png.get_height();

    let mut canvas = sdl::SDLWindow::new(width, height)?;

    let image_data = png.get_data();

    for h in 0..height {
        for w in 0..width {
            let pixel = &image_data[h as usize][w as usize];
            canvas.draw_point(pixel.to_color(), Point::new(w as i32, h as i32))?;
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
