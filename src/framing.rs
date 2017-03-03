use error::*;
use std::io::{Read, Write, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::FromPrimitive;
use method::EncodedMethod;

enum_from_primitive! {
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FrameType {
    METHOD = 1,
    HEADERS = 2,
    BODY  = 3,
    HEARTBEAT = 8
}
}

impl Copy for FrameType {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FramePayload(Vec<u8>);

impl FramePayload {
    pub fn new(data: Vec<u8>) -> Self {
        FramePayload(data)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EncodedProperties(Vec<u8>);

impl EncodedProperties {
    pub fn new(data: Vec<u8>) -> Self {
        EncodedProperties(data)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frame {
    pub frame_type: FrameType,
    pub channel: u16,
    pub payload: FramePayload,
}

pub struct FrameHeader {
    pub frame_type_id: u8,
    pub channel: u16,
    pub payload_size: u32,
}

impl FrameHeader {
    pub fn new(header: [u8; 7]) -> Self {
        let reader = &mut &header[..];
        let frame_type_id = reader.read_u8().unwrap();
        let channel = reader.read_u16::<BigEndian>().unwrap();
        let payload_size = reader.read_u32::<BigEndian>().unwrap();
        FrameHeader {
            frame_type_id: frame_type_id,
            channel: channel,
            payload_size: payload_size,
        }
    }
}


#[derive(Debug, Clone)]
pub struct MethodFrame {
    pub class_id: u16,
    pub method_id: u16,
    pub arguments: EncodedMethod,
}

impl MethodFrame {
    pub fn encode(&self) -> Result<FramePayload> {
        let mut writer = Vec::with_capacity(self.arguments.inner().len() + 4);
        try!(writer.write_u16::<BigEndian>(self.class_id));
        try!(writer.write_u16::<BigEndian>(self.method_id));
        try!(writer.write_all(self.arguments.inner()));
        Ok(FramePayload::new(writer))
    }

    // We need this method, so we can match on class_id & method_id
    pub fn decode(frame: &Frame) -> Result<MethodFrame> {
        if frame.frame_type != FrameType::METHOD {
            return Err(ErrorKind::Protocol("Not a method frame".to_string()).into());
        }
        let reader = &mut frame.payload.inner();
        let class_id = try!(reader.read_u16::<BigEndian>());
        let method_id = try!(reader.read_u16::<BigEndian>());
        let mut arguments = vec![];
        try!(reader.read_to_end(&mut arguments));
        Ok(MethodFrame {
            class_id: class_id,
            method_id: method_id,
            arguments: EncodedMethod::new(arguments),
        })
    }

    pub fn method_name(&self) -> &'static str {
        method_name(self)
    }

    pub fn carries_content(&self) -> bool {
        method_carries_content(self)
    }
}
include!("method_frame_methods.rs");


unsafe impl Send for Frame {}

impl Frame {
    pub fn decode<T: Read>(reader: &mut T) -> Result<Frame> {
        let mut header = [0u8; 7];
        try!(reader.read_exact(&mut header));
        let FrameHeader { frame_type_id, channel, payload_size } = FrameHeader::new(header);
        let size = payload_size as usize;
        // We need to use Vec because the size is not know in compile time.
        let mut payload: Vec<u8> = vec![0u8; size];
        try!(reader.read_exact(&mut payload));
        let frame_end = try!(reader.read_u8());
        if frame_end != 0xCE {
            return Err(ErrorKind::Protocol("Frame didn't end with 0xCE".to_string()).into());
        }
        let frame_type = match FrameType::from_u8(frame_type_id) {
            Some(ft) => ft,
            None => return Err(ErrorKind::Protocol("Unknown frame type".to_string()).into()),
        };

        let frame = Frame {
            frame_type: frame_type,
            channel: channel,
            payload: FramePayload::new(payload),
        };
        Ok(frame)
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Vec::with_capacity(self.payload.inner().len() + 8);
        try!(writer.write_u8(self.frame_type as u8));
        try!(writer.write_u16::<BigEndian>(self.channel));
        try!(writer.write_u32::<BigEndian>(self.payload.inner().len() as u32));
        try!(writer.write_all(self.payload.inner()));
        try!(writer.write_u8(0xCE));
        Ok(writer)
    }
}

#[derive(Debug, Clone)]
pub struct ContentHeaderFrame {
    pub content_class: u16,
    pub weight: u16,
    pub body_size: u64,
    pub properties_flags: u16,
    pub properties: EncodedProperties,
}

impl ContentHeaderFrame {
    pub fn decode(frame: &Frame) -> Result<ContentHeaderFrame> {
        let mut reader = Cursor::new(frame.payload.inner());
        let content_class = try!(reader.read_u16::<BigEndian>());
        let weight = try!(reader.read_u16::<BigEndian>()); //0 all the time for now
        let body_size = try!(reader.read_u64::<BigEndian>());
        let properties_flags = try!(reader.read_u16::<BigEndian>());
        let mut properties = vec![];
        try!(reader.read_to_end(&mut properties));
        Ok(ContentHeaderFrame {
            content_class: content_class,
            weight: weight,
            body_size: body_size,
            properties_flags: properties_flags,
            properties: EncodedProperties::new(properties),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = Vec::with_capacity(self.properties.inner().len() + 14);
        try!(writer.write_u16::<BigEndian>(self.content_class));
        try!(writer.write_u16::<BigEndian>(self.weight)); //0 all the time for now
        try!(writer.write_u64::<BigEndian>(self.body_size));
        try!(writer.write_u16::<BigEndian>(self.properties_flags));
        try!(writer.write_all(self.properties.inner()));
        Ok(writer)
    }
}

#[test]
fn test_encode_decode() {
    let frame = Frame {
        frame_type: FrameType::METHOD,
        channel: 5,
        payload: FramePayload::new(vec![1, 2, 3, 4, 5]),
    };
    let frame_encoded = frame.encode().ok().unwrap();
    assert_eq!(frame,
               Frame::decode(&mut Cursor::new(frame_encoded)).ok().unwrap());
}
