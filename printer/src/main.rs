use std::{
    io::*,
    path::PathBuf,
    sync::mpsc,
    sync::mpsc::Receiver,
    sync::mpsc::{TryRecvError},
    thread::{self, sleep},
    time::Duration,
    sync::{Arc,Mutex}
};

use clap::Parser;

use port_reader::PortReader;
use termios::*;

use common_protocols::opcode_protocol as op;
use defmt_printer_based_api as dpba;

mod base_protocol_handler;
use base_protocol_handler::BaseProtocolReader as bpr;
mod port_reader;

mod ser_port;
use ser_port::SerPort;

/// serial input and print program
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    port_name: String,

    /// the serial device baud rate
    baud: u32,

    /// Path to embedded program elf
    elf_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let port: SerPort = match serialport::new(args.port_name.clone(), args.baud)
        .timeout(Duration::from_millis(10))
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .open()
    {
        Ok(p) => {SerPort(Arc::new(Mutex::new(p)))},
        Err(e) => {
            panic!("Failed to open \"{}\". Error: {}", args.port_name, e);
        }
    };
    let (ser_tx, ser_rx) = mpsc::channel::<u8>();
    let (cin_tx, cin_rx) = mpsc::channel::<u8>();

    let ser_int = PortReader::new(SerPort(port.0.clone()),ser_tx, 1000);
    let cin_int = PortReader::new(stdin(),cin_tx, 1000);

    spawn_port_read_thread(ser_int, Duration::from_nanos(10));
    spawn_port_read_thread(cin_int, Duration::from_nanos(10));

    defmt_decoder::log::init_logger(false, false, move |metadata| {
        defmt_decoder::log::is_defmt_frame(metadata)
    });

    let log_helper = dpba::DefmtPrintHelper::new(args.elf_path).unwrap();

    let stdin_fd = 0;
    let termios = prep_tremios(stdin_fd);

    loop_logic(port, cin_rx, bpr::new(ser_rx), log_helper);

    tcsetattr(stdin_fd, TCSANOW, &termios).unwrap();
}

// Returns old the previous termios to allow the program to revert to the previous state
fn prep_tremios(fd: i32) -> Termios {
    let termios = Termios::from_fd(fd).unwrap();
    let mut termios_new = termios.clone();
    termios_new.c_lflag &= !(ICANON | ECHO);
    tcsetattr(fd, TCSANOW, &termios_new).unwrap();
    return termios;
}

fn spawn_port_read_thread<T: Read + std::marker::Send + 'static>(mut read: PortReader<T>, cooldown : Duration) {
    thread::spawn(move || {
        loop {
            match read.try_read() {
                Some(_) => {
                    // nice so there isn't much to do
                },
                None => {},
            }
            sleep(cooldown);
        }
    });
}

fn loop_logic(
    port : SerPort,
    cin_rx: Receiver<u8>,
    mut ser_in: bpr,
    mut log_helper: dpba::DefmtPrintHelper
) -> Option<()> {
    loop {
        match ser_in.try_read_frame() {
            Ok(frame) => {
                let op_frame = op::OpCode::from_slice(&frame).unwrap();
                handle_new_frame(op_frame.0, op_frame.1, &mut log_helper);
            },
            Err(base_protocol_handler::ReaderState::Broken) => {
                // if we reached a broken state nothing can be done and the program should close
                break;
            },
            Err(base_protocol_handler::ReaderState::INVALID) => {
                // send jam message
            }
            _ => {
                // there is nothing to do for incomplete frames
            }
        }
        handle_term(&cin_rx, &port)?;
    }
    None
}

fn handle_new_frame(
    opcode: op::OpCode,
    data: &[u8],
    log_helper: &mut dpba::DefmtPrintHelper,
) -> Option<()> {
    match opcode {
        op::OpCode::ECHO => {
            handle_echo_data(data).ok()?;
        }
        op::OpCode::LOG => {
            log_helper.handle_frame(data).ok()?;
            stdout().lock().flush().unwrap();
        }
        op::OpCode::JAM => {
            // we should stop sending data for some time
            return None;
        }
        _ => {
            // might be op::OpCode::INVALID
            // should never reach this stage
            return None;
        }
    }
    Some(())
}

fn handle_echo_data(data: &[u8]) -> std::result::Result<(), std::str::Utf8Error> {
    let str = std::str::from_utf8(data)?;
    print!("{}", str);
    stdout().lock().flush().unwrap();
    Ok(())
}

fn handle_term(term_rx: &Receiver<u8>, port: &SerPort) -> Option<()> {
    match term_rx.try_recv() {
        Ok(data) => {
            write_to_interface(base_protocol_handler::make_frame_from_slice(&data.to_le_bytes()[..1]).as_slice(), port).ok()
        },
        Err(TryRecvError::Empty) => {
            Some(())
        }
        Err(_) => {
            None
        }
    }
}

fn write_to_interface<T: Write>(data: &[u8], mut port: T) -> Result<()> {
    let mut wr = data;
    while !wr.is_empty() {
        let _ = match port.write(wr) {
            Ok(len) => {
                wr = &wr[len..];
                Ok(())
            }
            Err(e) => {
                if e.kind() == ErrorKind::TimedOut {
                    // should try again
                    continue;
                }
                Err(e)
            }
        };
    }
    Ok(())
}