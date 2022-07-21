use std::{error::Error, fmt::Debug};

use crate::{crc, filter::revert_filters};
use bitflags::bitflags;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use sdl2::pixels::Color;

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
enum Type {
    Ihdr = 0x49484452,
    Plte = 0x504c5445,
    Idat = 0x49444154,
    Iend = 0x49454e44,
}

struct Chunk<'a> {
    length: u32,
    typ: Type,
    data: &'a [u8],
    crc: u32,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct BigEndian4([u8; 4]);

impl BigEndian4 {
    fn get(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }
}

impl Debug for BigEndian4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

bitflags! {
    #[repr(transparent)]
    struct ColorType: u8 {
        const COLORTYPE_GRAYSCALE = 0;
        const COLORTYPE_PALETTE = 1;
        const COLORTYPE_COLOR = 2;
        const COLORTYPE_ALPHA_CHANNEL = 4;
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interlace {
    No = 0,
    Adam7 = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Compression {
    InflateDeflate = 0,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMethod {
    AdaptiveFiltering = 0,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ihdr {
    width: BigEndian4,
    height: BigEndian4,
    bit_depth: u8,
    colour_type: ColorType,
    compression_method: Compression,
    filter_method: FilterMethod,
    interlace_method: Interlace,
}

pub struct InProcessPng {
    ihdr: Option<Ihdr>,
    idat: Vec<u8>,
    scanlines_with_filter: Vec<Vec<u8>>,
}

impl Debug for InProcessPng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ihdr: {:?}, scanlines_with_filter: {} x {}",
            self.ihdr,
            self.scanlines_with_filter.len(),
            self.scanlines_with_filter
                .first()
                .unwrap_or(&Vec::new())
                .len()
        )
    }
}

impl InProcessPng {
    pub fn new() -> Self {
        Self {
            ihdr: None,
            idat: Vec::new(),
            scanlines_with_filter: Vec::new(),
        }
    }

    pub fn finalize(self) -> Result<Png, Box<dyn Error>> {
        construct(self)
    }

    pub fn get_scanlines(&self) -> &Vec<Vec<u8>> {
        &self.scanlines_with_filter
    }

    pub fn get_bpp(&self) -> usize {
        let ihdr = &self.ihdr.expect("ihdr must be available");
        match ihdr.colour_type {
            ColorType::COLORTYPE_COLOR => 3 * (ihdr.bit_depth as usize / 8),
            _ => todo!(),
        }
    }
}

pub struct Rgb(u8, u8, u8);

impl Rgb {
    pub fn to_color(&self) -> Color {
        Color::RGB(self.0, self.1, self.2)
    }
}

pub struct Png {
    width: u32,
    height: u32,
    data: Vec<Vec<Rgb>>,
}

impl Png {
    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_data(&self) -> &Vec<Vec<Rgb>> {
        &self.data
    }
}

const MAGIC_NUMBER: &[u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn construct(png: InProcessPng) -> Result<Png, Box<dyn Error>> {
    let resulting_data = revert_filters(&png).iter().map(to_rgb).collect();
    let ihdr = png.ihdr.ok_or("No IHDR present")?;
    Ok(Png {
        width: ihdr.width.get(),
        height: ihdr.height.get(),
        data: resulting_data,
    })
}

fn to_rgb(data: &Vec<u8>) -> Vec<Rgb> {
    data.chunks(3).map(|x| Rgb(x[0], x[1], x[2])).collect()
}

pub fn process(data: &[u8]) -> Result<Png, Box<dyn Error>> {
    let chunks = get_chunks(data)?;

    let mut png = InProcessPng::new();

    for chunk in chunks {
        println!("Length: {} Type: {:?}", chunk.length, chunk.typ);
        match chunk.typ {
            Type::Ihdr => process_ihdr(chunk, &mut png),
            Type::Idat => append_idat(chunk, &mut png),
            Type::Plte => todo!(),
            Type::Iend => {}
        }
    }

    uncompress_idat(&mut png)?;

    println!("Result: {:?}", png);

    png.finalize()
}

fn uncompress_idat(png: &mut InProcessPng) -> Result<(), String> {
    let deflated = inflate::inflate_bytes_zlib(&png.idat)?;
    let ihdr = &png.ihdr.expect("IHDR must be already parsed");
    let scanlinesize = 1 + match ihdr.colour_type {
        ColorType::COLORTYPE_COLOR => 3 * ihdr.width.get() * (ihdr.bit_depth as u32 / 8),
        _ => todo!(),
    } as usize;

    png.scanlines_with_filter = deflated.chunks(scanlinesize).map(|x| x.to_vec()).collect();

    Ok(())
}

fn append_idat(chunk: Chunk, png: &mut InProcessPng) {
    png.idat.extend_from_slice(chunk.data);
}

fn process_ihdr(chunk: Chunk, png: &mut InProcessPng) {
    let ihdr: Ihdr = convert(chunk.data);
    assert_eq!(ihdr.compression_method, Compression::InflateDeflate);
    assert_eq!(ihdr.filter_method, FilterMethod::AdaptiveFiltering);
    png.ihdr = Some(ihdr);
}

fn convert<T>(data: &[u8]) -> T {
    assert!(data.len() == std::mem::size_of::<T>());
    unsafe { std::ptr::read(data.as_ptr() as *const _) }
}

fn get_chunks(mut data: &'_ [u8]) -> Result<Vec<Chunk<'_>>, Box<dyn Error>> {
    assert!(data.len() > MAGIC_NUMBER.len());

    let magic_number_in_data = &data[..MAGIC_NUMBER.len()];
    data = &data[MAGIC_NUMBER.len()..];

    assert_eq!(
        MAGIC_NUMBER, magic_number_in_data,
        "Magic number does not match.\nExpected:\n{:02x?}\nGot:\n{:02x?}",
        MAGIC_NUMBER, magic_number_in_data
    );

    let mut chunks = Vec::new();

    while data.len() >= 12 {
        let length = u32::from_be_bytes(data[..4].try_into()?);
        let typ = u32::from_be_bytes(data[4..8].try_into()?);

        let typ = if let Some(typ) = FromPrimitive::from_u32(typ) {
            typ
        } else {
            data = &data[length as usize + 12..];
            continue;
        };

        let end_of_data = 8 + length as usize;

        assert!(data.len() >= 12 + length as usize);

        let inner_data = &data[8..end_of_data];
        let crc = u32::from_be_bytes(data[end_of_data..end_of_data + 4].try_into()?);

        let chunk = Chunk {
            length,
            typ,
            data: inner_data,
            crc,
        };

        let mut crc = crc::CrcCalculator::new();
        crc.update(&data[4..length as usize + 8]);
        let crc = crc.get_crc();

        assert_eq!(crc, chunk.crc, "CRC does not match");

        chunks.push(chunk);

        data = &data[length as usize + 12..];
    }

    Ok(chunks)
}
