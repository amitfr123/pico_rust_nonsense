use defmt::global_logger;
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section;

enum FrameState {
    VALID,
    INVALID
}

static mut FRAME_STATE: FrameState = FrameState::INVALID;
static mut CURRENT_FRAME: heapless::Vec<u8, 1024> = heapless::Vec::new();
static mut DEQUE: heapless::Deque::<heapless::Vec<u8, 1024>, 50> = heapless::Deque::new();
static DEQUE_STATE: AtomicBool = AtomicBool::new(false);
static mut RESTORE_STATE: critical_section::RestoreState = critical_section::RestoreState::invalid();
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[global_logger]
struct GlobLog;

unsafe impl defmt::Logger for GlobLog {
    fn acquire() {
        unsafe{
            RESTORE_STATE =  critical_section::acquire();
            CURRENT_FRAME.clear();
            FRAME_STATE = FrameState::VALID;
            ENCODER.start_frame(write_to_queue);
        }
        // now we disabled irq and can continue with our work
    }

    unsafe fn release() {
        unsafe{
            ENCODER.end_frame(write_to_queue);
            match FRAME_STATE {
                FrameState::VALID => {
                    let temp: heapless::Vec<u8, 1024> = CURRENT_FRAME.clone();
                    match DEQUE.push_back(temp) {
                        Ok(_) => {
                            // nice
                            DEQUE_STATE.store(true, Ordering::Relaxed);
                        },
                        Err(_) => {
                            // lame
                        }
                    }
                },
                _ => {
                    // bummer
                }
            }
            critical_section::release(RESTORE_STATE);
        }
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, write_to_queue);
    }

    unsafe fn flush() {
        // idk how to flush so poop
    }
}

fn write_to_queue(data: &[u8])
{   
    unsafe {
        match FRAME_STATE {
            FrameState::VALID => {
                data.into_iter().try_for_each(|f| Some({
                    match CURRENT_FRAME.push(*f) {
                        Ok(_) => {
                            // cool man
                        },
                        Err(_) => {
                            FRAME_STATE = FrameState::INVALID;
                            return None;
                        },
                    };
                }));
            },
            _ => {
                // bummer
            }
        }
    }
}

pub fn log_read() -> Option<heapless::Vec<u8, 1024>>{
    let mut data: Option<heapless::Vec<u8, 1024>> = None;
    if DEQUE_STATE.load(Ordering::Relaxed)
    {
        unsafe {
            RESTORE_STATE =  critical_section::acquire();
            match DEQUE.pop_front() {
                Some(p) => {
                    data = Some(p);
                },
                None => {
                    // lame
                },
            }
            if DEQUE.is_empty() {
                DEQUE_STATE.store(false, Ordering::Relaxed);
            }
            critical_section::release(RESTORE_STATE); 
        }
    }
    data
}