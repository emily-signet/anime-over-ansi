use super::metadata::{CompressionMode};



use simd_adler32::adler32;



use std::time::Duration;

/// Options for the encoder writing this packet into a sink / file.
#[derive(Copy, Clone, Debug)]
pub struct PacketFlags {
    pub compression_mode: CompressionMode,
    pub compression_level: Option<i32>,
    pub is_keyframe: bool,
}

/// The base-level data unit, representing a single frame of video or subtitle data.
#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Index of the stream this packet belongs to.
    pub stream_index: u32,
    /// Adler32 checksum of this packet's data
    pub checksum: u32,
    /// Length of this packet's data in bytes
    pub length: u64,
    /// Presentation time of packet
    pub time: Duration,
    /// Duration of packet, if available
    pub duration: Option<Duration>,
    /// Packet data
    pub data: Vec<u8>,
    /// Options for the encoder writing this packet into a sink / file.
    pub encoder_opts: Option<PacketFlags>,
}

impl EncodedPacket {
    /// Create a packet from data, adding in a calculated checksum.
    pub fn from_data(
        stream_index: u32,
        time: Duration,
        duration: Option<Duration>,
        data: Vec<u8>,
        encoder_opts: Option<PacketFlags>,
    ) -> EncodedPacket {
        EncodedPacket {
            time,
            duration,
            checksum: adler32(&data.as_slice()),
            length: data.len() as u64,
            encoder_opts,
            stream_index,
            data,
        }
    }

    /// Switch the data in the packet, optionally re-calculating the checksum.
    pub fn switch_data(&mut self, data: Vec<u8>, refresh_checksum: bool) {
        if refresh_checksum {
            self.checksum = adler32(&data.as_slice());
        }

        self.length = data.len() as u64;
        self.data = data;
    }
}

/// A transformer that takes an object and converts it into an [EncodedPacket] if possible; else returning none.
pub trait PacketTransformer<'a> {
    type Source;
    fn encode_packet(&mut self, src: &Self::Source) -> Option<EncodedPacket>;
}

/// A transformer that takes an [EncodedPacket] and converts it into an object if possible; else returning none.
pub trait PacketDecoder {
    type Output;
    fn decode_packet(&mut self, src: EncodedPacket) -> Option<Self::Output>;
}
