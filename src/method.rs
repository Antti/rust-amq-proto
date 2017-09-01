use framing::{FrameType, Frame, FramePayload, MethodFrame};
use error::Result;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EncodedMethod(Vec<u8>);

impl EncodedMethod {
    pub fn new(data: Vec<u8>) -> Self {
        EncodedMethod(data)
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn inner(&self) -> &[u8] {
        &self.0
    }
}

pub trait Method {
    fn decode(method_frame: MethodFrame) -> Result<Self> where Self: Sized;
    fn encode(&self) -> Result<EncodedMethod>;
    fn name(&self) -> &'static str;
    const ID: u16;
    const CLASS_ID: u16;

    fn encode_method_frame(&self) -> Result<FramePayload> {
        let frame = MethodFrame {
            class_id: Self::CLASS_ID,
            method_id: Self::ID,
            arguments: self.encode()?,
        };
        frame.encode()
    }

    fn to_frame(&self, channel: u16) -> Result<Frame> {
        Ok(Frame {
            frame_type: FrameType::METHOD,
            channel: channel,
            payload: self.encode_method_frame()?,
        })
    }
}
