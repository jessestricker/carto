use std::io;
use std::io::Read;

/// See [_Interface DataInput_, Java® Platform, Standard Edition & Java Development Kit Version 19 API Specification](<https://docs.oracle.com/en/java/javase/19/docs/api/java.base/java/io/DataInput.html#readFully(byte%5B%5D)>)
pub trait JavaRead: Read {
    fn read_byte(&mut self) -> io::Result<i8>;
    fn read_short(&mut self) -> io::Result<i16>;
    fn read_unsigned_short(&mut self) -> io::Result<u16>;
    fn read_int(&mut self) -> io::Result<i32>;
    fn read_long(&mut self) -> io::Result<i64>;

    fn read_float(&mut self) -> io::Result<f32>;
    fn read_double(&mut self) -> io::Result<f64>;

    fn read_utf(&mut self) -> io::Result<String>;
}

macro_rules! read_primitive_fn {
    ($name:ident,$type:ty) => {
        fn $name(&mut self) -> io::Result<$type> {
            let mut buf = [0u8; std::mem::size_of::<$type>()];
            self.read_exact(&mut buf)?;
            Ok(<$type>::from_be_bytes(buf))
        }
    };
}

impl<R: Read> JavaRead for R {
    read_primitive_fn! { read_byte, i8 }
    read_primitive_fn! { read_short, i16 }
    read_primitive_fn! { read_unsigned_short, u16 }
    read_primitive_fn! { read_int, i32 }
    read_primitive_fn! { read_long,  i64 }
    read_primitive_fn! { read_float, f32 }
    read_primitive_fn! { read_double, f64 }

    fn read_utf(&mut self) -> io::Result<String> {
        // https://github.com/openjdk/jdk/blob/030b071db1fb6197a2633a04b20aa95432a903bc/src/java.base/share/classes/java/io/DataInputStream.java#L561-L655

        let utf_length = self.read_unsigned_short()?;
        let mut bytes = vec![0u8; utf_length as usize];
        self.read_exact(&mut bytes)?;

        let mut bytes_iter = bytes.into_iter();
        let mut code_units: Vec<u16> = Vec::with_capacity(utf_length as usize); // UTF-16

        // reads the next continuation byte, strips the prefix and extends to u16
        fn next_cont_byte(bytes_iter: &mut std::vec::IntoIter<u8>) -> io::Result<u16> {
            let b = bytes_iter
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
            // 10xx_xxxx
            if b >> 6 == 0b10 {
                Ok((b & 0b0011_1111) as u16)
            } else {
                Err(io::ErrorKind::InvalidData.into())
            }
        }

        while let Some(b0) = bytes_iter.next() {
            if b0 >> 7 == 0b0 {
                // 0xxx_xxxx
                let b0 = (b0 & 0b0111_1111) as u16;
                code_units.push(b0);
            } else if b0 >> 5 == 0b110 {
                // 110x_xxxx 10xx_xxxx
                let b0 = (b0 & 0b0001_1111) as u16;
                let b1 = next_cont_byte(&mut bytes_iter)?;
                code_units.push(b0 << 6 | b1);
            } else if b0 >> 4 == 0b1110 {
                // 1110_xxxx 10xx_xxxx 10xx_xxxx
                let b0 = (b0 & 0b0000_1111) as u16;
                let b1 = next_cont_byte(&mut bytes_iter)?;
                let b2 = next_cont_byte(&mut bytes_iter)?;
                code_units.push(b0 << 12 | b1 << 6 | b2);
            } else {
                // invalid lead byte
                return Err(io::ErrorKind::InvalidData.into());
            }
        }

        String::from_utf16(&code_units).map_err(|_| io::ErrorKind::InvalidData.into())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn read_byte() {
        pub const DATA: [u8; 1] = [0xc0];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(-64, reader.read_byte().unwrap());
        assert_eq!(DATA.len(), reader.position() as usize);
    }

    #[test]
    fn read_short() {
        pub const DATA: [u8; 2] = [0xc0, 0x0];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(-16384, reader.read_short().unwrap());
        assert_eq!(DATA.len(), reader.position() as usize);
    }

    #[test]
    fn read_unsigned_short() {
        pub const DATA: [u8; 2] = [0x7f, 0xff];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(32767, reader.read_unsigned_short().unwrap());
        assert_eq!(DATA.len(), reader.position() as usize);
    }

    #[test]
    fn read_int() {
        pub const DATA: [u8; 4] = [0xc0, 0x0, 0x0, 0x0];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(-1073741824, reader.read_int().unwrap());
        assert_eq!(DATA.len(), reader.position() as usize);
    }

    #[test]
    fn read_long() {
        pub const DATA: [u8; 8] = [0xc0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(-4611686018427387904, reader.read_long().unwrap());
        assert_eq!(DATA.len(), reader.position() as usize);
    }

    #[test]
    fn read_utf() {
        pub const DATA: [u8; 10] = [0x0, 0x8, 0x41, 0xce, 0xbc, 0xc0, 0x80, 0xe1, 0x88, 0x9f];
        let mut reader = Cursor::new(&DATA);
        assert_eq!(
            "\u{0041}\u{03BC}\u{0000}\u{121F}",
            reader.read_utf().unwrap()
        );
        assert_eq!(DATA.len(), reader.position() as usize);
    }
}
