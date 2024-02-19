use std::io::Read;

use crc::Crc;
use crc::CRC_16_IBM_SDLC;
use crc::CRC_64_XZ;

/// 计算文件hash
pub fn calculate_hash(read: &mut impl Read) -> String {
    let crc64 = Crc::<u64>::new(&CRC_64_XZ);
    let mut crc64 = crc64.digest();
    
    let crc16 = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let mut crc16 = crc16.digest();
    
    let mut buffer = [0u8; 16 * 1024];

    loop {
        let count = read.read(&mut buffer).unwrap();

        if count == 0 {
            break;
        }

        crc64.update(&buffer[0..count]);
        crc16.update(&buffer[0..count]);
    }

    format!("{:016x}_{:04x}", &crc64.finalize(), crc16.finalize())
}