mod crc32;
mod filesystems;
mod partition_tables;

use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::partition_tables::{gpt::GPTHeader, msdos::MBR};
use clap::{Parser, Subcommand};

#[derive(Parser)]
/// Reading MBR partition table from a disk
struct Cli {
    /// Command to execute (mbr or gpt)
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    MBR {
        /// Path to the disk device (e.g., /dev/sda)
        disk_path: String,
    },
    GPT {
        /// Path to the disk device (e.g., /dev/sda)
        disk_path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Assuming we are running on a little-endian system allows us
    // to read structures directly from disk to memory without byte-swapping
    assert!(
        cfg!(target_endian = "little"),
        "Parsers in this tool only written for little-endian systems"
    );

    match cli.command {
        Command::MBR { disk_path } => {
            println!("Reading MBR partition table from {}", disk_path);
            let mbr = MBR::read(&disk_path).expect("Failed to read MBR from disk");
            println!("{:#?}", mbr);
        }
        Command::GPT { disk_path } => {
            println!("Reading GPT partition table from {}", disk_path);
            let gpt = GPTHeader::read(&disk_path).expect("Failed to read GPT from disk");
            println!("{:#?}", gpt);
        }
    }
}
