use std::{
    sync::{mpsc::Sender},
    io::{Read, ErrorKind},
};

#[derive(Debug, Clone)]
pub struct PortReader<T: Read> {
    output : Sender<u8>,
    port: T,
    buf : Vec<u8>,
}

impl<T: Read> PortReader<T> {
    pub fn new(port: T, output : Sender<u8>, read_size : usize) -> Self {
        PortReader {
            output : output,
            port: port,
            buf : vec![0; read_size],
        }
    }

    pub fn try_read(& mut self) -> Option<usize> {
        match self.port.read(self.buf.as_mut_slice()) {
            Ok(len) => {
                for byte in &self.buf.as_slice()[..len] {
                    self.output.send(*byte).ok()?
                }
                Some(len)
            },
            Err(e) => {
                if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                    Some(0)
                } else {
                    None
                }
            },
        }
    }
}
