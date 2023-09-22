use std::{
    sync::{Arc, Mutex},
    io::{Read, Write, Result},
};

use serialport::SerialPort;

// TODO: consider rw or some other locking because there is a good chance that reading doesn't effect writes
pub struct SerPort(pub Arc<Mutex<Box<dyn SerialPort>>>);

impl Read for SerPort {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.lock().unwrap().read(buf)
    }
}

impl Write for SerPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> { 
        self.0.lock().unwrap().flush()
    }
}

// TODO: find a better way to do this
impl Write for &SerPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> { 
        self.0.lock().unwrap().flush()
    }
}