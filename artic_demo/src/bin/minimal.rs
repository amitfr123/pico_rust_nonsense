#![no_main]
#![no_std]

use artic_demo as _;

#[rtic::app(
    device = artic_demo::hal::pac,
    dispatchers = [I2C0_IRQ],
    peripherals = true
)]
mod app {
    use usb_device::{class_prelude::*, prelude::*};
    use usbd_serial::SerialPort;
    
    use artic_demo::{
        hal::{
            self,
            clocks::init_clocks_and_plls,
            timer::{monotonic::Monotonic, Alarm0},
            watchdog::Watchdog,
            usb::UsbBus
        },
        XOSC_CRYSTAL_FREQ, pac::Interrupt
    };
    use base_protocol as bp;
    use core::{convert::TryFrom, mem::size_of};
    const MAX_COMMAND_LINE_LEN: usize = 64;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type MyMono = Monotonic<Alarm0>;

    #[shared]
    struct Shared {
        serial: SerialPort<'static, hal::usb::UsbBus>,
        usb_dev: UsbDevice<'static, UsbBus>
    }
    #[local]
    struct Local {
    }

    #[init(local = [usb_bus: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        unsafe {
            hal::sio::spinlock_reset();
        }
        let mut resets = cx.device.RESETS;
        let mut watchdog = Watchdog::new(cx.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mut timer = hal::Timer::new(cx.device.TIMER, &mut resets);
        let alarm = timer.alarm_0().unwrap();


        let usb_bus: &'static _ = cx.local.usb_bus.insert(UsbBusAllocator::new(hal::usb::UsbBus::new(
            cx.device.USBCTRL_REGS,
            cx.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut resets,
        )));
        
        let serial = SerialPort::new(usb_bus);

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(2)
            .build();

        rtic::pend(Interrupt::USBCTRL_IRQ);
        
        (
            Shared {
                serial,
                usb_dev
            },
            Local {
            },
            init::Monotonics(
                Monotonic::new(timer, alarm),
            ),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {        
        loop {
            match artic_demo::glob_log::log_read() {
                Some(data) => {
                    // cool
                    task1::spawn(data).ok().unwrap();
                },
                None => {
                    // lame
                }
            }
            continue;
        }
    }

    // TODO: Add tasks
    #[task(shared = [serial])]
    fn task1(cx: task1::Context, vec: heapless::Vec<u8, 1024>) {
        let mut serial = cx.shared.serial;
        (serial).lock(|serial| {
            write_serial_msg(serial, vec.as_slice(), bp::OpCode::LOG);
        });
    }

    #[task(binds = USBCTRL_IRQ, shared = [serial, usb_dev])]
    fn usb0(cx: usb0::Context) {
        let serial = cx.shared.serial;
        let usb_dev = cx.shared.usb_dev;
        match (serial, usb_dev).lock(try_receive_from_serial) {
            Some(data) => {
                // send to a software task to pop this data and handle it
            }
            None => {
                // Not much to do but consider adding error handling later
            }
        }
    }

    fn try_receive_from_serial(serial: &mut SerialPort<UsbBus>, usb_dev: &mut UsbDevice<UsbBus>) -> Option<heapless::Vec<u8, MAX_COMMAND_LINE_LEN>> {
        if usb_dev.poll(&mut [serial]) {
            let mut buf = [0u8; MAX_COMMAND_LINE_LEN];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    let mut data: heapless::Vec<u8, MAX_COMMAND_LINE_LEN> = heapless::Vec::new();
                    for e in (&buf[..count]).iter() {
                        data.push(*e).unwrap();
                    }
                    return Some(data);
                }
            }
        }
        None
    }

    fn handle_serial_letter(c: char, com_line: &mut heapless::String<MAX_COMMAND_LINE_LEN>, serial: &mut SerialPort<UsbBus>) {
        match c {
            '\n' => {
                let del: [u8; 1] = [b'n'];
                write_serial_msg(serial, &del[..1], bp::OpCode::ECHO);
                handle_command(com_line.clone());
                // todo handle com
                com_line.clear();
            },
            '\x08' => {
                let del: [u8; 3] = [b'\x08', b' ', b'\x08'];
                if com_line.len() > 0 {
                    write_serial_msg(serial, &del[..3], bp::OpCode::ECHO);
                }
            },
            _ => {

            },
        }
    }

    fn write_serial_msg(serial: &mut SerialPort<UsbBus>, data: &[u8], opcode: bp::OpCode) {
        let header = bp::BaseProtocolHeader {
            code: opcode,
            size: u32::try_from(data.len()).unwrap_or_else(|_e| {
                defmt::error!("invalid write size"); // should never happen
                0
            })
        };
        if header.size != 0 {
            // todo consider changing this h_buf to a static buffer
            let mut h_buf: [u8; size_of::<bp::BaseProtocolHeader>()] = [0; size_of::<bp::BaseProtocolHeader>()];
            // this unwrap is unnecessary but i like putting it just to show that this operation cannot fail
            header.into_slice(&mut h_buf[..size_of::<bp::BaseProtocolHeader>()]).unwrap();
            write_all_to_serial(serial, &h_buf[..size_of::<bp::BaseProtocolHeader>()]);
            write_all_to_serial(serial, data);
        }
    }

    fn write_all_to_serial(serial: &mut SerialPort<UsbBus>, data: &[u8]) {
        let mut wr_ptr = data;
        while !wr_ptr.is_empty() {
            let _ = serial.write(wr_ptr).map(|len| {
                wr_ptr = &wr_ptr[len..];
            });
        }
    }

    fn handle_command(com_line: heapless::String<MAX_COMMAND_LINE_LEN>) {
        // parse the command string and handle it
    }
}
