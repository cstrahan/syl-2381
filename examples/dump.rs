// use serial::prelude::*;
use std::time::Duration;

extern crate syl2381;
use syl2381::Syl2381;

fn main() {
    let port_name = "/dev/tty.usbserial-A10MMQO2";

    let port = serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .flow_control(serialport::FlowControl::None)
        .timeout(Duration::from_secs(3))
        .open()
        .expect("opening serial port");

    let mut pid = Syl2381::new(5, port);

    dump_params(&mut pid);
}

pub fn dump_params(pid: &mut Syl2381) {
    use paste::paste;

    macro_rules! print_holdings {
        ( $( $name:expr ),* ) => {
            paste! {
                $(
                    let val = pid.[< get_ $name:lower  >]();
                    println!("{: <7} = {}", stringify!($name), val);
                )*
            }
        };
    }

    println!("Dynamic Params");
    println!("==============");
    print_holdings!(PV, OUT);
    println!("{: <7} = {}", "AL1_STA", pid.get_al1_sta());
    print_holdings!(CV);
    println!("{: <7} = {:#?}", "AT", pid.get_at());

    println!();
    println!("Static Params");
    println!("=============");
    print_holdings!(
        SV, AH1, AL1, P, I, D, BB, SOUF, OT, FILT, INTY, OUTY, COTY, HY, PSB, RD, CORF, ID, BAUD
    );
}

// mod embedded_serial {

//     use std::io;

//     use embedded_hal::serial::{ErrorKind, ErrorType};

//     use crate::SerialPort;

//     #[derive(Debug, Copy, Clone)]
//     pub struct SerialError {
//         kind: io::ErrorKind,
//     }

//     impl embedded_hal::serial::Error for SerialError {
//         fn kind(&self) -> ErrorKind {
//             #[allow(clippy::match_single_binding)]
//             match self.kind {
//                 _ => ErrorKind::Other,
//             }
//         }
//     }

//     impl From<io::Error> for SerialError {
//         fn from(e: io::Error) -> Self {
//             SerialError { kind: e.kind() }
//         }
//     }

//     impl ErrorType for Box<dyn SerialPort> {
//         type Error = SerialError;
//     }

//     mod nonblocking {
//         use super::*;
//         use embedded_hal_nb::serial;
//         use serial::unix::TTYPort;

//         pub struct EmbeddedSerial {
//             pub serial: TTYPort,
//         }

//         fn io_error_to_nb(err: io::Error) -> nb::Error<SerialError> {
//             match err.kind() {
//                 io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => nb::Error::WouldBlock,
//                 other => nb::Error::Other(SerialError { kind: other }),
//             }
//         }

//         impl serial::Read<u8> for Box<dyn SerialPort> {
//             fn read(&mut self) -> nb::Result<u8, Self::Error> {
//                 let mut buffer = [0; 1];
//                 let bytes_read = io::Read::read(self, &mut buffer).map_err(io_error_to_nb)?;
//                 if bytes_read > 0 {
//                     Ok(buffer[0])
//                 } else {
//                     Err(nb::Error::WouldBlock)
//                 }
//             }
//         }

//         impl serial::Write<u8> for Box<dyn SerialPort> {
//             fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
//                 io::Write::write(self, &[word])
//                     .map_err(io_error_to_nb)
//                     .map(|_| ())
//             }

//             fn flush(&mut self) -> nb::Result<(), Self::Error> {
//                 io::Write::flush(self).map_err(io_error_to_nb)
//             }
//         }
//     }

//     mod blocking {
//         use super::*;
//         use embedded_hal::serial;

//         impl serial::Write<u8> for Box<dyn SerialPort> {
//             fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
//                 Ok(io::Write::write_all(self, buffer)?)
//             }

//             fn flush(&mut self) -> Result<(), Self::Error> {
//                 Ok(io::Write::flush(self)?)
//             }
//         }
//     }
// }
