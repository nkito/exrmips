#![allow(dead_code)]
extern crate log;
extern crate simplelog;

use simplelog::*;
use std::io;
use std::io::prelude::*;
use std::fs::File;
//use std::{thread, time::Duration};
use log::{info,error};

use exrmips1::{exrmips, SPIFlashParam};
use clap::{arg, command, value_parser};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let matches = command!() // requires `cargo` feature
    .arg(arg!(
        -d --debug  "Turn debugging information on"
    ))
    .arg(
        arg!(
            -b --breakpoint [addr]  "Enable breakpoint"
        ).required(false)
        .value_parser(value_parser!(String)),
    )
    .arg(
        arg!(
            -r --run [insts]   "Specifies number of instruction execusions after break (in hexadecimal)"
        ).required(false)
        .value_parser(value_parser!(String)),
    )
    .arg(
        arg!(
            -f --flash [capacity]   "Specifies size of flash memory size"
        ).required(false)
        .value_parser(value_parser!(u32)),
    )
    .arg(
        arg!(
            [FILE] "System image file"
        )
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
    .get_matches();


    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("exrmips.log").unwrap()),
        ]
    ).unwrap();

    let mut flash_param:&SPIFlashParam = &exrmips::SPI_FLASH_PARAM_S25FL164K;

    if let Some(flash_capacity) = matches.get_one::<u32>("flash") {
        match flash_capacity {
            256 => {
                flash_param = &exrmips::SPI_FLASH_PARAM_MX66U2G45G;
                info!("Flash size = 256 MB");
            }
            _ => {
                flash_param = &exrmips::SPI_FLASH_PARAM_S25FL164K;
                info!("Flash size = 8 MB");
            }
        }
    }
    let mut flashdata = vec![0xff as u8; flash_param.capacity as usize].into_boxed_slice();

    if let Some(file_path) = matches.get_one::<PathBuf>("FILE") {
        match File::open(file_path) {
            Ok( mut f) => {
                f.read(&mut flashdata)?;
            }
            _ => {
                error!("Can not open image file \"{}\"", file_path.as_os_str().to_str().unwrap());
            }
        }
    }

    let bindata = &flashdata[0..flashdata.len()];

    let mut ms = exrmips::generate_machine_state(flash_param,bindata);

    if let Some(breakpoint_str) = matches.get_one::<String>("breakpoint") {
        match u32::from_str_radix(breakpoint_str, 16) {
            Ok(addr) => {
                ms.emu.breakpoint = addr;
                info!("Breakpoint is enabled : 0x{:x}", addr);
            }
            _ => {
                error!("Breakpoint \"{}\" is incorrect and is ignored", breakpoint_str);
            }
        }
    }

    if let Some(run_str) = matches.get_one::<String>("run") {
        match u32::from_str_radix(run_str, 16) {
            Ok(ninstr) => {
                ms.emu.runafterbreak = ninstr as u64;
                info!("#instruction after break : 0x{:x}", ms.emu.runafterbreak);
            }
            _ => {
                error!("The specified number of instructions after break \"{}\" is incorrect and is ignored", run_str);
            }
        }
    }

    exrmips::run_term(&mut ms);

    Ok(())
}
