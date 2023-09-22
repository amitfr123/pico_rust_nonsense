/*
   This module implements the behavior for the base protocol
*/

use std::{
    sync::mpsc::{Receiver, TryRecvError},
    result::Result,
    time
};

use common_protocols::base_protocol as bp;

#[derive(Debug, Clone, Copy)]
pub enum ReaderState {
    Broken,
    INVALID,
    INCOMPLETE
}

#[derive(Debug)]
pub struct BaseProtocolReader {
    frame: Vec<u8>,
    byte_stream: Receiver<u8>,
    state: ReaderState,
    invalid_time : time::Instant
}

impl BaseProtocolReader {
    pub fn new(byte_stream: Receiver<u8>) -> Self {
        BaseProtocolReader {
            frame: Vec::new(),
            byte_stream: byte_stream,
            state: ReaderState::INCOMPLETE,
            invalid_time : time::Instant::now()
        }
    }

    pub fn try_read_frame(&mut self) -> Result<Vec<u8>, ReaderState> {
        match self.state {
            ReaderState::Broken => {
                // once we reach this state nothing can be done
                Err(ReaderState::Broken)
            },
            ReaderState::INVALID => {
                if self.invalid_time.elapsed().as_secs() < bp::INVALID_STATE_COOLDOWN.try_into().unwrap() {
                    self.empty_byte_reader();
                    Err(self.state)
                }
                else {
                    self.state = ReaderState::INCOMPLETE;
                    self.try_build_frame()  
                }
            },
            ReaderState::INCOMPLETE => {
                self.try_build_frame()  
            },
        }
    }

    fn try_build_frame(&mut self) -> Result<Vec<u8>, ReaderState> {
        loop {
            match self.byte_stream.try_recv() {
                Ok(byte) => {
                    self.frame.push(byte);
                    match bp::try_from_frame(self.frame.as_slice()) {
                        Ok(slice) => {
                            let mut v : Vec<u8> = Vec::new();
                            v.extend_from_slice(slice);
                            self.frame.clear();
                            return Ok(v);
                        },
                        Err(bp::BaseProtocolLayerError::INVALID) => {
                            self.state = ReaderState::INVALID;
                            self.frame.clear();
                            self.invalid_time = time::Instant::now();
                            return Err(self.state);
                        },
                        _ => {
                            // this means we are still incomplete
                        },
                    }
                },
                Err(e) => {
                    if e == TryRecvError::Disconnected {
                        self.state = ReaderState::Broken;
                    }
                    return Err(self.state);
                }
            }
        }
    }

    fn empty_byte_reader(&mut self) {
        loop {
            match self.byte_stream.try_recv() {
                Ok(_) => {
                    // reset the timer
                    self.invalid_time = time::Instant::now();
                },
                Err(e) => {
                    if e == TryRecvError::Disconnected {
                        self.state = ReaderState::Broken;
                    }
                    break;
                }
            }
        }
    }
}

pub fn make_frame_from_slice(slice : &[u8]) -> Vec<u8> {
    let mut v : Vec<u8> = Vec::new();
    v.resize(slice.len(),0);
    bp::into_frame(v.as_mut_slice(), slice).unwrap();
    v
}