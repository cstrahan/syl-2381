use serial::prelude::*;
use std::time::Duration;

extern crate syl2381;
use syl2381::Syl2381;

fn main() {
    let port = "/dev/tty.usbserial-A10MMQO2";
    let mut port = serial::open(port).unwrap();

    port.reconfigure(&|settings| {
        (settings.set_baud_rate(serial::Baud9600).unwrap());
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .expect("port conf");

    port.set_timeout(Duration::from_secs(3600))
        .expect("port timeout");

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
