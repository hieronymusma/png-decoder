use std::error::Error;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::crc;

#[repr(u32)]
#[derive(Debug, FromPrimitive, PartialEq, Eq)]
enum Type {
    IHDR = 0x49484452,
    PLTE = 0x504c5445,
    IDAT = 0x49444154,
    IEND = 0x49454e44,
}

struct Chunk<'a> {
    length: u32,
    typ: Type,
    data: &'a [u8],
    crc: u32,
}

struct InProcessPng {
    width: Option<u32>,
    height: Option<u32>,
}

impl InProcessPng {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
        }
    }
}

const MAGIC_NUMBER: &[u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

pub fn process(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let chunks = get_chunks(data)?;

    let mut png = InProcessPng::new();

    for chunk in chunks {
        println!("Length: {} Type: {:?}", chunk.length, chunk.typ);
        match chunk.typ {
            Type::IHDR => process_ihdr(chunk, &mut png),
            Type::IDAT => {}
            Type::PLTE => {}
            Type::IEND => {}
        }
    }

    Ok(())
}

fn process_ihdr(chunk: Chunk, png: &mut InProcessPng) {}

fn get_chunks<'a>(mut data: &'a [u8]) -> Result<Vec<Chunk<'a>>, Box<dyn Error>> {
    assert!(data.len() > MAGIC_NUMBER.len());

    let magic_number_in_data = &data[..MAGIC_NUMBER.len()];
    data = &data[MAGIC_NUMBER.len()..];

    assert_eq!(
        MAGIC_NUMBER, magic_number_in_data,
        "Magic number does not match.\nExpected:\n{:02x?}\nGot:\n{:02x?}",
        MAGIC_NUMBER, magic_number_in_data
    );

    let mut chunks = Vec::new();

    while data.len() > 0 {
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
