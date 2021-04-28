use serde::{Serialize, Deserialize};
use std::{fs, io, env, path};
use epson::sent_print;
use std::io::prelude::*;



#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub inputs: Vec<String>,
    pub output: String,
}

pub fn get_config() -> io::Result<Config> {
    let mut output = String::new();
    for i in fs::read_dir("/dev/usb").unwrap() {
        let entry = i.unwrap().path();
        let str = entry.file_name().unwrap().to_str().unwrap();
        if str.contains("lp") {
            output = format!("/dev/usb/{}", str);
        };
    }
    let config_path = get_path("config.json");
    let config = fs::read_to_string(config_path)?;
    let mut out: Config = serde_json::from_str(&config)?;
    out.output = output;
    Ok(out)
}


pub fn add_log(conten: &str) {
    let log_apth = get_path("err_log.log");
    let mut file = fs::OpenOptions::new()
        .append(true).open(log_apth).unwrap();
    file.write(format!("{}{}\n", " ", conten).as_bytes()).unwrap();
}

pub fn get_path(file_name: &str) -> path::PathBuf {
    let start_dir = match env::args().nth(0) {
        Some(v) => v,
        _ => {
            sent_print("none dir", "/dev/usb/lp0").unwrap();
            panic!("none dir");
        },
    };
    let path = match path::Path::new(&start_dir).parent() {
        Some(v) => v,
        _ => {
            sent_print("none dir", "/dev/usb/lp0").unwrap();
            panic!("none dir");
        },
    };
    let mut path_b = path::PathBuf::from(path);
    path_b.push(file_name);
    path_b
}

