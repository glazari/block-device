mod filesystems;
mod partition_tables;

use crate::partition_tables::msdos::MBR;
use clap::Parser;

#[derive(Parser)]
/// Reading MBR partition table from a disk
struct Cli {
    /// Path to the disk device (e.g., /dev/sda)
    disk_path: String,
}

fn main() {
    let cli = Cli::parse();

    let mbr = MBR::read(&cli.disk_path);
    let mbr = mbr.expect("Failed to read MBR");
    println!("{:#?}", mbr);
}
