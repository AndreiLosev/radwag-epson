mod config;

use std::io::prelude::*;
use serial::prelude::*;
use std::{io, str, result::Result, time::Duration, error::Error};
use epson::sent_print;

fn main() -> Result<(), Box<dyn Error>> {
    let ports_path = match config::get_config() {
        Ok(v) => v,
        Err(e) => {
            sent_print(&err_mes(e.to_string()), "/dev/usb/lp0").unwrap();
            config::add_log(&err_mes(e.to_string()));
            panic!("{}", e);
        },
    }; 
    let mut ports = ports_path.inputs.iter()
        .map(|i| match create_port(i) {
            Ok(port) => port,
            Err(e) => {
                sent_print(&err_mes(e.to_string()), &ports_path.output).unwrap();
                config::add_log(&err_mes(e.to_string()));
                panic!("{}", e);
            }
        }).collect::<Vec<serial::SystemPort>>();
    match run(&mut ports, &ports_path.output) {
        Ok(v) => v,
        Err(e) => {
            sent_print(&err_mes(e.to_string()), &ports_path.output).unwrap();
            config::add_log(&err_mes(e.to_string()));
            panic!("{}", e);
        }
    };
    Ok(())
}

pub fn create_port(port_path: &str) -> io::Result<serial::SystemPort> {
    let mut port = serial::open(port_path)?;
        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_parity(serial::ParityNone);
            Ok(())
        })?;
    port.set_timeout(Duration::from_millis(100))?;
    Ok(port)
}

pub fn run(ports: &mut [serial::SystemPort], output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut buff = [0_u8; 1024];
    let mut string_buff = String::new();
    loop {
        for port in ports.iter_mut() {

            let n = match port.read(&mut buff) {
                Ok(n) => n,
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::TimedOut => continue,
                        _ => return Err(Box::new(e)),
                    }
                }
            };
            let text = buff_to_str(&buff[0..n])?;
            string_buff.push_str(&text);
            if ((buff[n - 1] == 8) || (buff[n - 1] == 10)) & (string_buff.len() > 15) {
                match sent_print(&string_buff, output_path)  {
                    Ok(v) => v,
                    Err(e) => {
                        config::add_log(&err_mes(e.to_string()));
                        continue
                    }
                };
                string_buff = String::new();
            } else if string_buff.len() > 200 {
                string_buff = String::new();
            }

        }
        buff = [0_u8; 1024];
    }
}


fn err_mes(mes: String) -> String {
    format!("it's a fucking error \n Err message => {}\nreboot me and try again", mes)
}

fn buff_to_str(buf: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let vec_buf = buf.iter().filter(|&i| i.is_ascii())
        .filter(|&i| i != &32)
        .map(|i| *i).collect::<Vec<u8>>();
    let text = String::from_utf8(vec_buf)?;
    let patterns = ["Date", "Time", "Net", "Tare", "BalanceID", "Gross", "]g", "re"];
    for i in patterns.iter() {
        if text.contains(i) {
            return Ok(text);
        }
    }
    Ok(String::new())
}
