/*
   This protocol is used to pass data over a stream styled connection.
   This protocol was created to be as simple as possible and is meant to wrap around other protocols.

   The protocol structure:
   [preamble][size][data][trailer]

   Endian: be
   preamble: u16 = 0xabcd
   size: u32
   trailer: u16 = 0x1234
*/
use core::mem::size_of;

#[derive(Debug)]
pub enum BaseProtocolLayerError {
    INCOMPLETE,
    INVALID,
}

pub struct BaseProtocolLayer;

pub const INVALID_STATE_COOLDOWN : usize = 2;

const FRAME_PREAMBLE : u16 = 0xabcd;
const FRAME_TRAILING : u16 = 0x1234;

const FRAME_INFO_SIZE : usize = size_of::<u16>() * 2 + size_of::<u32>();
pub const MAX_FRAME_SIZE : usize = 1024;
pub const MAX_DATA_SIZE : usize = MAX_FRAME_SIZE - FRAME_INFO_SIZE;

impl ad::AdditiveProtocol<u32, BaseProtocolLayerError> for BaseProtocolLayer {
    // attempt to read the header and data from slice
    fn from_slice(slice: &[u8]) -> Result<(u32,&[u8]), BaseProtocolLayerError> {
        if slice.len() < FRAME_INFO_SIZE {
            Err(BaseProtocolLayerError::INCOMPLETE)
        } else {
            let mut frame = slice;

            // preamble check
            if u16::from_be_bytes(frame[..size_of::<u16>()].try_into().unwrap()) != FRAME_PREAMBLE {
                return Err(BaseProtocolLayerError::INVALID);
            }
            
            frame = &frame[size_of::<u16>()..];
            let size = u32::from_be_bytes(frame[..size_of::<u32>()].try_into().unwrap());
            if size > MAX_DATA_SIZE.try_into().unwrap() {
                return Err(BaseProtocolLayerError::INVALID);
            }
            // check if we hold the rest of the data + the trailing
            else if usize::try_from(size).unwrap() + size_of::<u16>() > slice.len() {
                return Err(BaseProtocolLayerError::INCOMPLETE);
            }
            frame = &frame[size_of::<u32>()..];
    
            // trailing check
            if u16::from_be_bytes(frame[usize::try_from(size).unwrap()..].try_into().unwrap()) != FRAME_TRAILING {
                return Err(BaseProtocolLayerError::INVALID);
            }
            Ok((size, &frame[..usize::try_from(size).unwrap()]))
        }
    }

    // write the header into the slice
    fn into_frame(slice: &mut [u8], header: u32) -> Result<&[u8], BaseProtocolLayerError> {
        if slice.len() > MAX_FRAME_SIZE || slice.len() < FRAME_INFO_SIZE {
            Err(BaseProtocolLayerError::INVALID)
        } else {
            let mut t_slice = slice;
            t_slice[..size_of::<u16>()].copy_from_slice(&u16::to_be_bytes(FRAME_PREAMBLE));
            t_slice = &mut t_slice[size_of::<u16>()..];
            t_slice[..size_of::<u32>()].copy_from_slice(&u32::to_be_bytes(header));
            t_slice = &mut t_slice[size_of::<u32>()..];
            t_slice[usize::try_from(header).unwrap()..size_of::<u16>()].copy_from_slice(&u16::to_be_bytes(FRAME_TRAILING));
            Ok(&t_slice[..usize::try_from(header).unwrap()])
        }
    }

    // return a slice with the remaining space for the protocol
    fn reserve_header(slice: &[u8]) -> Option<&[u8]> {
        if slice.len() < FRAME_INFO_SIZE {
            None
        } else {
            Some(&slice[size_of::<u16>() + size_of::<u32>()..slice.len() - size_of::<u16>()])
        }
    }
}

impl BaseProtocolLayer {
    pub fn get_reserve_size() -> usize {
        FRAME_INFO_SIZE
    }
}