mod filesystems;
mod partition_tables;

use std::{fs::File, io::Read};

use crate::partition_tables::msdos::MBR;

fn main() {
    let mut buffer = [0u8; 512];
    let disk = "/dev/sda";

    let mbr = MBR::read(disk).expect("Failed to read MBR");
    println!("{:#?}", mbr);
}
