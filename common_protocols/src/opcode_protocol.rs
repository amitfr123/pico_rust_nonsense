/*
   This protocol is used to pass data over a byte stream styled connection (read 1 byte at a time).
   This protocol was created to be as simple as possible and is meant to wrap around other protocols.

   The protocol structure:
   [preamble][size][data][trailer]

   Endian: be
   preamble: u16 = 0xabcd
   size: u32
   trailer: u16 = 0x1234
*/
use core::mem::size_of;

macro_rules! get_repr {
    () => {};
    (#[repr($ty:ty)] $($tail:tt)*) => {
        $ty
    };
    (#[$meta:meta] $($tail:tt)*) => {
        get_repr!($($tail)*)
    };
}

macro_rules! normal {
    ($ty:ty; $(#[$meta:meta])* $vis:vis enum $name:ident {
        $($fn:ident = $val:expr,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($fn = $val,)*
        }
        impl core::convert::TryFrom<$ty> for $name {
            type Error = ();

            fn try_from(v: $ty) -> Result<Self, Self::Error> {
                match v {
                    $($val => Ok($name::$fn),)*
                    _ => Err(()),
                }
            }
        }

        // this could be replaced with a simple as but this approach forces you to use the correct type
        impl core::convert::From<$name> for $ty {
            fn from(v: $name) -> Self {
                v as $ty
            }
        }
    };
}

macro_rules! meta_magic {
    () => {};
    ($($tts:tt)*) => {
        normal!(get_repr!($($tts)*); $($tts)*);
    };
}
meta_magic! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(u16)]
    pub enum OpCode {
        INVALID = 0,
        ECHO = 1,
        LOG = 2,
        JAM = 0xffff,
    }
}

pub const OPCODE_HEADER_SIZE : usize = size_of::<u16>();

impl OpCode {
    pub fn from_slice(slice: &[u8]) -> Option<(OpCode,&[u8])> {
        if slice.len() < size_of::<OpCode>() {
            None
        }
        else {
            match u16::from_le_bytes(
                slice[..size_of::<OpCode>()]
                    .try_into()
                    .unwrap(),
            )
            .try_into()
            {
                Ok(c) => Some((c, &slice[size_of::<OpCode>()..])),
                Err(_) => Some((OpCode::INVALID, &slice[size_of::<OpCode>()..])),
            }
        }
    }
}