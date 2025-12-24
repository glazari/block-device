mod filesystems;
mod partition_tables;

use std::{fs::File, io::Read};

fn main() {
    let mut buffer = [0u8; 512];
    let disk = "/dev/sda";
    let mut disk_fd = File::open(disk).expect("Failed to open disk");

    println!("Disk opened successfully: {:?}", disk_fd);
    disk_fd
        .read_exact(buffer.as_mut_slice())
        .expect("Failed to read disk data");

    println!("Read 512 bytes from disk:\n{:?}", &buffer[..]);
    let stringify = String::from_utf8_lossy(&buffer);
    println!("Data as string:\n{}", stringify);
}
