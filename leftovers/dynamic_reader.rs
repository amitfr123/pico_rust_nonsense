use std::cell::RefCell;
use std::io::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct DynReader<T: Read> {
    data: Vec<u8>,
    size: usize,
    offset: usize,
    interface: Rc<RefCell<T>>,
}

impl<T: Read> DynReader<T> {
    pub fn new(interface: Rc<RefCell<T>>) -> Self {
        DynReader {
            interface: interface,
            data: Vec::new(),
            size: 0,
            offset: 0,
        }
    }

    pub fn set_new_target_size(&mut self, size: usize) {
        self.data.clear();
        self.data.resize(size, 0);
        self.size = size;
        self.offset = 0;
    }

    pub fn try_read_frame(&mut self) -> Result<Vec<u8>> {
        let mut slice = self.data.as_mut_slice();
        slice = &mut slice[self.offset..self.size];
        let mut interface = self.interface.borrow_mut(); // todo: replace with check
        match interface.read(slice) {
            Ok(len) => {
                self.offset += len;
                if self.offset == self.size {
                    self.offset = 0;
                    Ok(self.data.clone())
                } else {
                    Err(Error::new(ErrorKind::WouldBlock, "frame isn't complete"))
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock {
                    Err(Error::new(ErrorKind::WouldBlock, "frame isn't complete"))
                } else {
                    Err(e)
                }
            }
        }
    }
}
