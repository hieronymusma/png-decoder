pub struct CrcCalculator {
    crc: u32,
}

impl CrcCalculator {
    pub fn new() -> Self {
        Self { crc: 0xffffffff }
    }

    pub fn get_crc(self) -> u32 {
        (self.crc ^ 0xffffffff) as u32
    }

    pub fn update(&mut self, data: &[u8]) {
        for byte in data {
            self.update_byte(*byte);
        }
    }

    fn update_byte(&mut self, data: u8) {
        self.crc = CrcCalculator::lookup(self.crc as u8 ^ data) ^ (self.crc >> 8);
    }

    fn lookup(data: u8) -> u32 {
        let mut c = data as u32;
        for _ in 0..8 {
            if (c & 1) > 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c = c >> 1;
            }
        }
        c
    }
}
