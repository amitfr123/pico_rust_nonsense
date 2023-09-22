/**
 * This file should allow for an easy way to implement a net stack.
 * Each layer will have its own implementation but some things should always remain the same so shared behavior is defined here
 */



#[derive(Debug)]
pub enum NetworkError {
    INCOMPLETE,
    INVALID,
    UNKNOWN,
    TIMEOUT,
    LINK_DOWN,
    INVALID_SIZE,
}

#[derive(Debug)]
pub struct IoNode {
    data : Option<&[u8]>,
    next : Option<IoNode>,
}

/**
 * This function should receive an IoNode that contains data.
 * Each layer can expend this list with its own data
 */
type PushFunc = fn(&IoNode) -> Option<NetworkError>;

/**
 * 
 * This function should receive an IoNode that contains data that pervious layer data field.
 * By doing this we allow the next layer to directly get their relevant information and also be able to check
 * the previous layer info if needed
 */
type PopFunc = fn(&IoNode) -> Option<NetworkError>;

pub struct BasicNetworkLayer {
    pub push : Option<PushFunc>,    
    pub pop : Option<PopFunc>,
}


// pub trait AdditiveProtocol<H, E> {
//     // attempt to read the header and data from slice
//     fn from_slice(slice: &[u8]) -> Result<(H,&[u8]), E>;

//     // write the header into the slice
//     fn into_frame(slice: &mut [u8], header: H) -> Result<&[u8], E>;

//     // return a slice with the remaining space for the protocol
//     fn reserve_header(slice: &[u8]) -> Option<&[u8]>;
// }

// pub struct ProtocolChain {
//     next : Option<&'static ProtocolChain>,
//     pub reserve_size : usize,
//     get_func: fn(&Container, &str) -> i32
// }

// impl ProtocolChain {
//     pub fn new(reserve_size : usize) -> ProtocolChain {
//         ProtocolChain {
//             next : None,
//             reserve_size : reserve_size
//         }
//     }

//     pub fn get_min_size_for_chain(&self) -> usize {
//         self.reserve_size + if self.next.is_some() { self.next.unwrap().get_min_size_for_chain() } else { 0 }
//     }
// }

// #[allow(dead_code)]
// pub const BP_NODE : ProtocolChain = ProtocolChain {
//     next: Some(&OP_NODE),
//     reserve_size: op::OPCODE_HEADER_SIZE
// };

// pub const OP_NODE : ProtocolChain = ProtocolChain {
//     next: None,
//     reserve_size: bp::FRAME_INFO_SIZE
// };
