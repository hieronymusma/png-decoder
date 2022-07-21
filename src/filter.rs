use num::FromPrimitive;
use num_derive::FromPrimitive;

use crate::png::InProcessPng;

#[repr(u8)]
#[derive(Debug, FromPrimitive)]
enum ScanLineFilterType {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

pub fn revert_filters(png: &InProcessPng) -> Vec<Vec<u8>> {
    let scanlines_with_filter = png.get_scanlines();
    let mut result: Vec<Vec<u8>> = scanlines_with_filter
        .iter()
        .map(|x| x[1..].to_vec())
        .collect();

    for scanline in 0..result.len() {
        let filter_type: ScanLineFilterType =
            FromPrimitive::from_u8(scanlines_with_filter[scanline][0])
                .expect("Filter type has to be valid");
        match filter_type {
            ScanLineFilterType::None => continue,
            ScanLineFilterType::Sub => filter_sub(&mut result, scanline, png),
            ScanLineFilterType::Up => filter_up(&mut result, scanline),
            ScanLineFilterType::Average => filter_average(&mut result, scanline, png),
            ScanLineFilterType::Paeth => filter_paeth(&mut result, scanline, png),
        }
    }
    result
}

fn filter_sub(data: &mut Vec<Vec<u8>>, scanline_index: usize, png: &InProcessPng) {
    let bpp = png.get_bpp();
    let scanline = &mut data[scanline_index];
    for b_idx in bpp..scanline.len() {
        scanline[b_idx] = ((scanline[b_idx] as usize + scanline[b_idx - bpp] as usize) % 256) as u8;
    }
}

fn filter_up(data: &mut Vec<Vec<u8>>, scanline_index: usize) {
    if scanline_index == 0 {
        return;
    }
    let prior = data[scanline_index - 1].clone();
    for (val, prior_element) in data[scanline_index].iter_mut().zip(prior) {
        *val = ((*val as usize + prior_element as usize) % 256) as u8;
    }
}

fn filter_average(data: &mut Vec<Vec<u8>>, scanline_index: usize, png: &InProcessPng) {
    let bpp = png.get_bpp();
    let scanline_size = data[scanline_index].len();
    let prior = if scanline_index > 0 {
        data[scanline_index - 1].clone()
    } else {
        vec![0; scanline_size]
    };
    let scanline = &mut data[scanline_index];
    for b_idx in 0..scanline_size {
        let before = if b_idx < bpp {
            0
        } else {
            scanline[b_idx - bpp] as usize
        };
        scanline[b_idx] = scanline[b_idx]
            + (((((before + prior[b_idx] as usize) as f64) / 2.0) as usize) % 256) as u8;
    }
}

fn filter_paeth(data: &mut Vec<Vec<u8>>, scanline_index: usize, png: &InProcessPng) {
    let bpp = png.get_bpp();
    let scanline_size = data[scanline_index].len();
    let prior = if scanline_index > 0 {
        data[scanline_index - 1].clone()
    } else {
        vec![0; scanline_size]
    };
    let scanline = &mut data[scanline_index];
    for b_idx in 0..scanline_size {
        let before = if b_idx < bpp {
            0
        } else {
            scanline[b_idx - bpp] as i64
        };
        let before_prior = if b_idx < bpp {
            0
        } else {
            prior[b_idx - bpp] as i64
        };

        scanline[b_idx] = ((scanline[b_idx] as usize
            + paeth_predictor(before, prior[b_idx] as i64, before_prior))
            % 256) as u8;
    }
}

fn paeth_predictor(a: i64, b: i64, c: i64) -> usize {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    if pa <= pb && pa <= pc {
        a as usize
    } else if pb <= pc {
        b as usize
    } else {
        c as usize
    }
}

// function PaethPredictor (a, b, c)
// begin
//      ; a = left, b = above, c = upper left
//      p := a + b - c        ; initial estimate
//      pa := abs(p - a)      ; distances to a, b, c
//      pb := abs(p - b)
//      pc := abs(p - c)
//      ; return nearest of a,b,c,
//      ; breaking ties in order a,b,c.
//      if pa <= pb AND pa <= pc then return a
//      else if pb <= pc then return b
//      else return c
// end
