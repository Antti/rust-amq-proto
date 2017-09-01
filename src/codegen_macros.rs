use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bit_vec::BitVec;
use std::io::{Cursor, Read, Write};

use table::{Table, decode_table, encode_table};
use error::*;

#[derive(Debug)]
pub struct ArgumentsReader<'data> {
    cursor: Cursor<&'data [u8]>,
    bits: BitVec,
    byte: u8,
    current_bit: u8,
}

impl<'data> ArgumentsReader<'data> {
    pub fn new(data: &'data [u8]) -> ArgumentsReader {
        ArgumentsReader {
            cursor: Cursor::new(data),
            bits: BitVec::from_bytes(&[0]),
            byte: 0,
            current_bit: 0,
        }
    }

    pub fn read_octet(&mut self) -> Result<u8> {
        self.current_bit = 0;
        self.cursor.read_u8().map_err(From::from)
    }

    pub fn read_long(&mut self) -> Result<u32> {
        self.current_bit = 0;
        self.cursor.read_u32::<BigEndian>().map_err(From::from)
    }

    pub fn read_longlong(&mut self) -> Result<u64> {
        self.current_bit = 0;
        self.cursor.read_u64::<BigEndian>().map_err(From::from)
    }

    pub fn read_short(&mut self) -> Result<u16> {
        self.current_bit = 0;
        self.cursor.read_u16::<BigEndian>().map_err(From::from)
    }

    pub fn read_shortstr(&mut self) -> Result<String> {
        self.current_bit = 0;
        let size = self.read_octet()? as usize;
        let mut buffer: Vec<u8> = vec![0u8; size];
        self.cursor.read(&mut buffer[..])?;
        Ok(String::from_utf8_lossy(&buffer[..]).to_string())
    }

    pub fn read_longstr(&mut self) -> Result<String> {
        self.current_bit = 0;
        let size = self.read_long()? as usize;
        let mut buffer: Vec<u8> = vec![0u8; size];
        self.cursor.read(&mut buffer[..])?;
        Ok(String::from_utf8_lossy(&buffer[..]).to_string())
    }

    pub fn read_table(&mut self) -> Result<Table> {
        self.current_bit = 0;
        decode_table(&mut self.cursor).map(|(table, table_size)| table)
    }

    pub fn read_timestamp(&mut self) -> Result<u64> {
        self.current_bit = 0;
        self.read_longlong()
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        if self.current_bit == 0 || self.current_bit == 8 {
            self.current_bit = 0;
            self.byte = self.read_octet()?;
            self.bits = BitVec::from_bytes(&[self.byte]);
        }
        self.current_bit += 1;
        self.bits
            .get(7 - (self.current_bit - 1) as usize)
            .ok_or(ErrorKind::Protocol("Bitmap is not correct".to_owned()).into())
    }
}

#[derive(Debug)]
pub struct ArgumentsWriter {
    data: Vec<u8>,
    bits: BitVec,
    current_bit: u8,
}

impl ArgumentsWriter {
    pub fn new() -> Self {
        ArgumentsWriter {
            data: vec![],
            bits: BitVec::from_bytes(&[0]),
            current_bit: 0,
        }
    }

    pub fn write_octet(&mut self, data: &u8) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u8(*data).map_err(From::from)
    }

    pub fn write_long(&mut self, data: &u32) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u32::<BigEndian>(*data).map_err(From::from)
    }

    pub fn write_longlong(&mut self, data: &u64) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u64::<BigEndian>(*data).map_err(From::from)
    }

    pub fn write_short(&mut self, data: &u16) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u16::<BigEndian>(*data).map_err(From::from)
    }

    pub fn write_shortstr(&mut self, data: &String) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u8(data.len() as u8)?;
        self.data.write_all(data.as_bytes())?;
        Ok(())
    }

    pub fn write_longstr(&mut self, data: &String) -> Result<()> {
        self.flush_bits()?;
        self.data.write_u32::<BigEndian>(data.len() as u32)?;
        self.data.write_all(data.as_bytes())?;
        Ok(())
    }

    // Always a last method, since it writes to the end
    pub fn write_table(&mut self, data: &Table) -> Result<()> {
        self.flush_bits()?;
        encode_table(&mut self.data, &data)
    }

    pub fn write_timestamp(&mut self, data: &u64) -> Result<()> {
        self.flush_bits()?;
        self.write_longlong(data)
    }

    // TODO: Flush bytes on all subsequent other type of data writes
    pub fn write_bit(&mut self, data: &bool) -> Result<()> {
        self.bits.set(7 - self.current_bit as usize, *data);
        self.current_bit += 1;
        if self.current_bit == 7 {
            self.flush_bits();
        }
        Ok(())
    }

    pub fn flush_bits(&mut self) -> Result<()> {
        if self.current_bit > 0 {
            let res = self.data.write_all(&self.bits.to_bytes()).map_err(From::from);
            self.bits = BitVec::from_bytes(&[0]);
            self.current_bit = 0;
            res
        } else {
            Ok(())
        }
    }

    pub fn as_bytes(mut self) -> Vec<u8> {
        self.flush_bits();
        self.data
    }
}


macro_rules! map_type {
    (octet) => (u8);
    (long) => (u32);
    (longlong) => (u64);
    (short) => (u16);
    (shortstr) => (String);
    (longstr) => (String);
    (table) => (Table);
    (timestamp) => (u64);
    (bit) => (bool);
}

macro_rules! read_type {
    ($reader:expr, octet) => ($reader.read_octet());
    ($reader:expr, long) => ($reader.read_long());
    ($reader:expr, longlong) => ($reader.read_longlong());
    ($reader:expr, short) => ($reader.read_short());
    ($reader:expr, shortstr) => ($reader.read_shortstr());
    ($reader:expr, longstr) => ($reader.read_longstr());
    ($reader:expr, table) => ($reader.read_table());
    ($reader:expr, timestamp) => ($reader.read_timestamp());
    ($reader:expr, bit) => ($reader.read_bit());
}

macro_rules! write_type {
    ($writer:expr, octet, $data:expr) => ($writer.write_octet($data));
    ($writer:expr, long, $data:expr) => ($writer.write_long($data));
    ($writer:expr, longlong, $data:expr) => ($writer.write_longlong($data));
    ($writer:expr, short, $data:expr) => ($writer.write_short($data));
    ($writer:expr, shortstr, $data:expr) => ($writer.write_shortstr($data));
    ($writer:expr, longstr, $data:expr) => ($writer.write_longstr($data));
    ($writer:expr, table, $data:expr) => ($writer.write_table($data));
    ($writer:expr, timestamp, $data:expr) => ($writer.write_timestamp($data));
    ($writer:expr, bit, $data:expr) => ($writer.write_bit($data));
}

macro_rules! method_struct {
    ($method_name:ident, $method_str:expr, $class_id:expr, $method_id:expr, ) => (
        #[derive(Debug, PartialEq, Clone)]
        pub struct $method_name;
        impl method::Method for $method_name {
            const ID: u16 = $method_id;
            const CLASS_ID: u16 = $class_id;

            fn decode(_method_frame: MethodFrame) -> Result<Self> where Self: Sized {
                Ok($method_name)
            }

            fn encode(&self) -> Result<method::EncodedMethod> {
                Ok(method::EncodedMethod::new(vec![]))
            }

            fn name(&self) -> &'static str {
                $method_str
            }
        }
    );
    ($method_name:ident, $method_str:expr, $class_id:expr, $method_id:expr, $($arg_name:ident => $ty:ident),+) => (
        #[derive(Debug, PartialEq)]
        pub struct $method_name {
            $(pub $arg_name: map_type!($ty),)*
        }

        impl method::Method for $method_name {
            const ID: u16 = $method_id;
            const CLASS_ID: u16 = $class_id;

            fn decode(method_frame: MethodFrame) -> Result<Self> where Self: Sized {
                debug!("Decoding {}", $method_str);
                match (method_frame.class_id, method_frame.method_id) {
                    ($class_id, $method_id) => {},
                    _ => return Err(ErrorKind::Protocol("Unexpected method method class and id".to_string()).into())
                }
                let data = method_frame.arguments.into_inner();
                let mut reader = ArgumentsReader::new(&data);
                Ok($method_name {
                    $($arg_name: read_type!(reader, $ty)?,)*
                })
            }

            fn encode(&self) -> Result<method::EncodedMethod> {
                let mut writer = ArgumentsWriter::new();
                $(write_type!(writer, $ty, &self.$arg_name)?;)*
                Ok(method::EncodedMethod::new(writer.as_bytes()))
            }

            fn name(&self) -> &'static str {
                $method_str
            }
        }
    )
}

macro_rules! properties_struct {
    ($struct_name:ident, $($arg_name:ident => $ty:ident),+) => (
        #[derive(Debug, Default, PartialEq, Clone)]
        pub struct $struct_name {
            $(pub $arg_name: Option<map_type!($ty)>,)*
        }

        impl $struct_name {
            pub fn decode(content_header_frame: ContentHeaderFrame) -> Result<$struct_name> {
                let mut reader = ArgumentsReader::new(content_header_frame.properties.inner());
                let properties_flags = BitVec::from_bytes(&[((content_header_frame.properties_flags >> 8) & 0xff) as u8,
                    (content_header_frame.properties_flags & 0xff) as u8]);
                let mut idx = 0;
                Ok($struct_name {
                    $($arg_name: {
                        idx = idx + 1;
                        match properties_flags.get(idx - 1) {
                            Some(flag) if flag => Some(read_type!(reader, $ty)?),
                            Some(_) => None,
                            None => return Err(ErrorKind::Protocol("Properties flags are not correct".to_owned()).into())
                        }
                    },)*
                })
            }

            pub fn encode(self) -> Result<Vec<u8>> {
                let mut writer = ArgumentsWriter::new();
                $(if let Some(prop) = self.$arg_name {
                        write_type!(writer, $ty, &prop)?;
                };)*
                Ok(writer.as_bytes())
            }

            pub fn flags(&self) -> u16 {
                let mut bits = BitVec::from_elem(16, false);
                let mut idx = 0;
                $(
                    bits.set(idx, self.$arg_name.is_some());
                    idx = idx + 1;
                )*
                let flags : u16 = bits.to_bytes()[0] as u16;
                (flags << 8 | bits.to_bytes()[1] as u16) as u16
            }
        }
    );
}

#[cfg(test)]
mod test {
    use bit_vec::BitVec;
    use error::Result;
    use framing::{MethodFrame, ContentHeaderFrame};
    use super::*;
    use method::{self, Method, EncodedMethod};

    method_struct!(Foo, "test.foo", 1, 2, a => octet, b => shortstr, c => longstr, d => bit, e => bit, f => long);
    method_struct!(FooNoFields, "test.foo_no_fields", 1, 2, );

    properties_struct!(Test, a => octet, b => shortstr, c => longstr, d => bit, e => bit, f => long);

    #[test]
    fn test_encoding() {
        let f = Foo {
            a: 1,
            b: "test".to_string(),
            c: "bar".to_string(),
            d: false,
            e: true,
            f: 0xDEADBEEF,
        };
        assert_eq!(f.encode().unwrap().into_inner(),
                   vec![
            1, // 1
            4, // "test".len()
            116, 101, 115, 116, // "test"
            0, 0, 0, 3, // "bar".len()
            98, 97, 114, // "bar"
            2, // false, true => 0b00000010
            0xDE, 0xAD, 0xBE, 0xEF, // 0xDEADBEEF
        ]);
    }

    #[test]
    fn test_decoding() {
        let f = Foo {
            a: 1,
            b: "test".to_string(),
            c: "bar".to_string(),
            d: false,
            e: true,
            f: 0xDEADBEEF,
        };
        let frame = MethodFrame {
            class_id: 1,
            method_id: 2,
            arguments: EncodedMethod::new(vec![
            1, // 1
            4, // "test".len()
            116, 101, 115, 116, // "test"
            0, 0, 0, 3, // "bar".len()
            98, 97, 114, // "bar"
            2, // false, true => 0b00000010
            0xDE, 0xAD, 0xBE, 0xEF, // 0xDEADBEEF
        ]),
        };
        assert_eq!(Foo::decode(frame).unwrap(), f);
    }

    #[test]
    fn test_decoding_wrong_ids() {
        let f = Foo {
            a: 1,
            b: "test".to_string(),
            c: "bar".to_string(),
            d: false,
            e: true,
            f: 0xDEADBEEF,
        };
        let frame = MethodFrame {
            class_id: 42,
            method_id: 55,
            arguments: EncodedMethod::new(vec![
            1, // 1
            4, // "test".len()
            116, 101, 115, 116, // "test"
            0, 0, 0, 3, // "bar".len()
            98, 97, 114, // "bar"
            2, // false, true => 0b00000010
            0xDE, 0xAD, 0xBE, 0xEF, // 0xDEADBEEF
        ]),
        };
        assert_eq!(Foo::decode(frame).is_err(), true);
    }
}
