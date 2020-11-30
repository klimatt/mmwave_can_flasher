use socketcan::{CANSocket, CANFrame};
use clap::{Arg, App};
use std::{
    fs::File,
    path::Path,
    io::{Read},
    time::Duration,
};


fn main(){
    let matches = App::new("mmwave_can_flasher")
        .version("0.1.0")
        .author("Matvei Klimov <klimatt.gu@gmail.com>")
        .about("CAN-BUS flasher for ti mmwave radar with modified SBL on it")
        .arg(Arg::new("file.bin")
            .short("f".parse().unwrap())
            .long("file")
            .takes_value(true))
        .arg(Arg::new("radar_id")
            .short("r".parse().unwrap())
            .long("radar")
            .takes_value(true))
        .arg(Arg::new("can_bus_iface")
            .short("i".parse().unwrap())
            .long("iface")
            .takes_value(true))
        .get_matches();
    println!("mmwave_can_flasher starts!");

    let bin_path = Path::new(matches.value_of("file.bin").unwrap());

    let radar_id: u32 = matches.value_of("radar_id").unwrap().parse::<u32>().expect("Error");
    let can_iface = matches.value_of("can_bus_iface").unwrap();

    let mut file = match File::open(&bin_path) {
        Err(why) => panic!("Couldn't open {}: {}", bin_path.display(), why),
        Ok(file) => {
            println!("Successfully opened {}", bin_path.display());
            file
        },
    };
    let mut image_byte_array: Vec<u8> = Vec::new();

    match file.read_to_end(image_byte_array.as_mut()) {
        Err(why) => panic!("Couldn't read {}: {}", bin_path.display(), why),
        Ok(_) => println!("Successfully read {:}[bytes]", image_byte_array.len()),
    }

    let cs = match CANSocket::open(can_iface){
        Err(why) => panic!("Couldn't open {}: {}", can_iface, why),
        Ok(cs) => {
            println!("Successfully opened {}", can_iface);
            cs
        },
    };
    let read_timeout = Duration::new(50,0);
    match cs.set_read_timeout(read_timeout){
        Err(why) => panic!("Couldn't set timeout {}: {}", read_timeout.as_secs(), why),
        Ok(_) => println!("Successfully set timeout {}", read_timeout.as_secs()),
    }

    println!("Waiting start byte from SBL for {}[secs]...", read_timeout.as_secs());

    let read_frame : CANFrame = match cs.read_frame(){
        Err(why) => panic!("Nothing received from radar[{}] in {}[secs]: {}", radar_id, read_timeout.as_secs(), why),
        Ok(frame) => {
            println!("Received start frame from radar[{}]", frame.id());
            frame
        },
    };

    println!("Send command to prepare for image upload to radar[{}]", read_frame.id());
    let mut cmd_frame = CANFrame::new(radar_id, &[0x01,0x02,0x03,0x04], false, false).unwrap();
    cmd_frame.force_extended();

    match cs.write_frame(&cmd_frame){
        Err(why) => panic!("Can't send cmd to radar[{}] in {}[secs]: {}", radar_id, read_timeout.as_secs(), why),
        Ok(_) => println!("Successfully send  prepare cmd to radar[{}]", radar_id),

    }
}