// use serial::prelude::*;
use std::time::Duration;

extern crate syl2381;
use syl2381::Syl2381;

use eh_nb_1_0_alpha as embedded_hal;

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

    let port = embedded_serial::EmbeddedSerial { port: port };

    let mut pid = Syl2381::new(5, port);

    dump_params(&mut pid);
}

pub fn dump_params<S>(pid: &mut Syl2381<S>)
where
    S: embedded_hal::serial::Read<u8> + embedded_hal::serial::Write<u8>,
{
    use paste::paste;

    macro_rules! print_params {
        ( $( $name:expr ),* ) => {
            paste! {
                $(
                    let val = pid.[< get_ $name:lower  >]();
                    if let Ok(v) = val {
                        println!("{: >19} = {}", stringify!($name), v);
                    }
                )*
            }
        };
    }

    print_params!(
        pv,
        out,
        j1_status,
        cv,
        status,
        sv,
        j1_on_temp,
        j1_off_temp,
        p,
        i,
        d,
        bb,
        souf,
        control_cycle,
        filter,
        input_sensor_type,
        output_mode,
        output_type,
        hysteresis,
        input_offset,
        control_direction,
        display_unit,
        unit_id,
        baud_rate
    );
}

// An embedded_hal wrapper for serialport.
// See "Add optional support for embedded-hal traits" https://github.com/serialport/serialport-rs/pull/59
mod embedded_serial {
    pub struct EmbeddedSerial {
        pub port: Box<dyn SerialPort>,
    }

    use std::io;

    use eh1_0_alpha::serial::{ErrorKind, ErrorType};

    use serialport::SerialPort;

    #[derive(Debug, Copy, Clone)]
    pub struct SerialError {
        kind: io::ErrorKind,
    }

    impl eh1_0_alpha::serial::Error for SerialError {
        fn kind(&self) -> ErrorKind {
            #[allow(clippy::match_single_binding)]
            match self.kind {
                _ => ErrorKind::Other,
            }
        }
    }

    impl From<io::Error> for SerialError {
        fn from(e: io::Error) -> Self {
            SerialError { kind: e.kind() }
        }
    }

    impl ErrorType for EmbeddedSerial {
        type Error = SerialError;
    }

    mod nonblocking {
        use super::*;
        use eh_nb_1_0_alpha::serial;

        fn io_error_to_nb(err: io::Error) -> nb::Error<SerialError> {
            match err.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => nb::Error::WouldBlock,
                other => nb::Error::Other(SerialError { kind: other }),
            }
        }

        impl serial::Read<u8> for EmbeddedSerial {
            fn read(&mut self) -> nb::Result<u8, Self::Error> {
                let mut buffer = [0; 1];
                let bytes_read =
                    io::Read::read(&mut self.port, &mut buffer).map_err(io_error_to_nb)?;
                if bytes_read > 0 {
                    Ok(buffer[0])
                } else {
                    Err(nb::Error::WouldBlock)
                }
            }
        }

        impl serial::Write<u8> for EmbeddedSerial {
            fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
                io::Write::write(&mut self.port, &[word])
                    .map_err(io_error_to_nb)
                    .map(|_| ())
            }

            fn flush(&mut self) -> nb::Result<(), Self::Error> {
                io::Write::flush(&mut self.port).map_err(io_error_to_nb)
            }
        }
    }

    mod blocking {
        use super::*;
        use eh1_0_alpha::serial;

        impl serial::Write<u8> for EmbeddedSerial {
            fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
                Ok(io::Write::write_all(&mut self.port, buffer)?)
            }

            fn flush(&mut self) -> Result<(), Self::Error> {
                Ok(io::Write::flush(&mut self.port)?)
            }
        }
    }
}
