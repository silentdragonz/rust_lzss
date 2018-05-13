//! Decompresses LZSS types implemented by Nintendo based on https://github.com/magical/nlzss

#![feature(iterator_step_by)]
extern crate bit_vec;
extern crate byteorder;

use bit_vec::BitVec;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Error, ErrorKind, Read, Seek};
use std::mem::transmute;

/// Parses an LZSS header and data block returning the decompressed result
///
/// Assumes LZSS is constructed properly
///
/// # Panics
/// Will panic if the size in the data header is incorrect
///
/// # Errors
/// Returns an Error if the type is invalid or the size is wrong
///
/// # Examples
///
/// ```
/// use rust_lzss::decompress;
/// use std::io::Cursor;
///
/// let lzss10: [u8; 11] = [ 0x10, 0x14, 0x00, 0x00, 0x08, 0x61, 0x62, 0x63, 0x64, 0xD0, 0x03, ]; // abcdabcdabcdabcdabcdabcd
/// let decoded = decompress(&mut Cursor::new(lzss10));
/// ```
pub fn decompress<T: Read + Seek>(data: &mut T) -> Result<Vec<u8>, Error> {
    let lz_type = data.read_u8()?;
    let mut data_size_tmp: [u8; 4] = [0; 4];
    data.read_exact(&mut data_size_tmp[0..3])?;
    let data_size = unsafe { transmute::<[u8; 4], u32>(data_size_tmp) };

    match lz_type {
        0x10 => decompress_lzss10(data, data_size as usize),
        0x11 => decompress_lzss11(data, data_size as usize),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid header")),
    }
}

fn decompress_lzss10<T: Read + Seek>(data: &mut T, size: usize) -> Result<Vec<u8>, Error> {
    let mut decompress_data: Vec<u8> = Vec::new();
    let disp_extra = 1;
    while decompress_data.len() < size {
        let b = data.read_u8()?;
        let bits = BitVec::from_bytes(&[b]);
        for bit in bits.iter() {
            if bit {
                let val = data.read_u16::<BigEndian>()?;
                let count = (val >> 0xC) + 3;
                let disp = (val & 0xFFF) + disp_extra;
                for _ in 0..count {
                    let len = decompress_data.len();
                    let copy_data = decompress_data[len - disp as usize];
                    decompress_data.push(copy_data);
                }
            } else {
                decompress_data.push(data.read_u8()?);
            }

            if size <= decompress_data.len() {
                break;
            }
        }
    }

    if size <= decompress_data.len() {
        Ok(decompress_data)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            "Decompressed size does not match expected size.",
        ))
    }
}

fn decompress_lzss11<T: Read + Seek>(data: &mut T, size: usize) -> Result<Vec<u8>, Error> {
    let mut decompress_data: Vec<u8> = Vec::new();
    while decompress_data.len() < size {
        let b = data.read_u8()?;
        let bits = BitVec::from_bytes(&[b]);
        for bit in bits.iter() {
            if bit {
                let mut val = data.read_u8()?;
                let indicator = val >> 4;
                let mut count: u16;
                match indicator {
                    0 => {
                        count = u16::from(val) << 4;
                        val = data.read_u8()?;
                        count += u16::from(val) >> 4;
                        count += 0x11
                    }
                    1 => {
                        count = (u16::from(val) & 0xF) << 12;
                        val = data.read_u8()?;
                        count += u16::from(val) << 4;
                        val = data.read_u8()?;
                        count += u16::from(val) >> 4;
                        count += 0x111;
                    }
                    _ => {
                        count = indicator.into();
                        count += 1;
                    }
                }
                let mut disp: u16 = (u16::from(val) & 0xF) << 8;
                val = data.read_u8()?;
                disp += u16::from(val) + 1;

                for _ in 0..count {
                    let len = decompress_data.len();
                    let copy_data = decompress_data[len - disp as usize];
                    decompress_data.push(copy_data);
                }
            } else {
                decompress_data.push(data.read_u8()?);
            }

            if size <= decompress_data.len() {
                break;
            }
        }
    }

    if size <= decompress_data.len() {
        Ok(decompress_data)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            "Decompressed size does not match expected size.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn decompresses() {
        let lzss10: [u8; 11] = [
            0x10, 0x14, 0x00, 0x00, 0x08, 0x61, 0x62, 0x63, 0x64, 0xD0, 0x03,
        ]; // abcdabcdabcdabcdabcdabcd
        let lzss11: [u8; 11] = [
            0x11, 0x14, 0x00, 0x00, 0x08, 0x61, 0x62, 0x63, 0x64, 0xF0, 0x03,
        ]; // abcdabcdabcdabcdabcdabcd

        let mut result_lzss10: Vec<u8> = Vec::new();
        let mut result_lzss11: Vec<u8> = Vec::new();
        for _ in (0..20).step_by(4) {
            result_lzss10.push(0x61);
            result_lzss10.push(0x62);
            result_lzss10.push(0x63);
            result_lzss10.push(0x64);

            result_lzss11.push(0x61);
            result_lzss11.push(0x62);
            result_lzss11.push(0x63);
            result_lzss11.push(0x64);
        }

        assert_eq!(result_lzss10, decompress(&mut Cursor::new(lzss10)).unwrap());
        assert_eq!(result_lzss11, decompress(&mut Cursor::new(lzss11)).unwrap());
    }

    #[test]
    fn decompresses_lzss10() {
        let test1: [u8; 1] = [0x00];
        let test2: [u8; 9] = [0x00, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68]; // abcdefgh
        let test3: [u8; 7] = [0x08, 0x61, 0x62, 0x63, 0x64, 0xD0, 0x03]; // abcdabcdabcdabcdabcdabcd
        let mut result3: Vec<u8> = Vec::new();
        for _ in (0..20).step_by(4) {
            result3.push(0x61);
            result3.push(0x62);
            result3.push(0x63);
            result3.push(0x64);
        }
        assert_eq!(
            0,
            decompress_lzss10(&mut Cursor::new(test1), 0).unwrap().len()
        );
        assert_eq!(
            &[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68],
            decompress_lzss10(&mut Cursor::new(test2), 8)
                .unwrap()
                .as_slice()
        );
        assert_eq!(
            result3,
            decompress_lzss10(&mut Cursor::new(test3), 20).unwrap()
        );
    }

    #[test]
    fn decompresses_lzss11() {
        let test1: [u8; 1] = [0x00];
        let test2: [u8; 9] = [0x00, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68]; // abcdefgh
        let test3: [u8; 7] = [0x08, 0x61, 0x62, 0x63, 0x64, 0xF0, 0x03]; // abcdabcdabcdabcdabcdabcd
        let test4: [u8; 8] = [0x08, 0x61, 0x62, 0x63, 0x64, 0x01, 0x30, 0x03]; // abcdabcdabcdabcdabcdabcd
        let test5: [u8; 9] = [0x08, 0x61, 0x62, 0x63, 0x64, 0x10, 0x07, 0xB0, 0x03]; // abcdabcdabcdabcdabcdabcd
        let mut result3: Vec<u8> = Vec::new();
        let mut result4: Vec<u8> = Vec::new();
        let mut result5: Vec<u8> = Vec::new();

        for _ in (0..20).step_by(4) {
            result3.push(0x61);
            result3.push(0x62);
            result3.push(0x63);
            result3.push(0x64);
        }
        for _ in (0..40).step_by(4) {
            result4.push(0x61);
            result4.push(0x62);
            result4.push(0x63);
            result4.push(0x64);
        }
        for _ in (0..400).step_by(4) {
            result5.push(0x61);
            result5.push(0x62);
            result5.push(0x63);
            result5.push(0x64);
        }

        assert_eq!(
            0,
            decompress_lzss11(&mut Cursor::new(test1), 0).unwrap().len()
        );
        assert_eq!(
            &[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68],
            decompress_lzss11(&mut Cursor::new(test2), 8)
                .unwrap()
                .as_slice()
        );
        assert_eq!(
            result3,
            decompress_lzss11(&mut Cursor::new(test3), 20).unwrap()
        );
        assert_eq!(
            result4,
            decompress_lzss11(&mut Cursor::new(test4), 40).unwrap()
        );
        assert_eq!(
            result5,
            decompress_lzss11(&mut Cursor::new(test5), 400).unwrap()
        );
    }
}
