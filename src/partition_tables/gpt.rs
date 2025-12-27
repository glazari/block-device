//! GUID Partition Table (GPT)
//!
//! It is part of the Unified Extensible Firmware Interface (UEFI) standard.
//!
//! Supports more than 4 primary partitions on a disk.
//! Supports disks larger than 2 TB.

use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read, Seek},
};

use crate::partition_tables::msdos::{MBR, PartitionType};

// Invariants
const _: () = assert!(
    std::mem::size_of::<GPTHeader>() == 512,
    "GPTHeader size must be 512 bytes"
);

const SIGNATURE: &str = "EFI PART";
const SIG_LITTLE_ENDIAN: [u8; 8] = [0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54];
const SIG_BIG_ENDIAN: [u8; 8] = [0x54, 0x52, 0x41, 0x50, 0x20, 0x49, 0x46, 0x45];

#[repr(C)]
#[derive(Clone)]
pub struct GPTHeader {
    pub signature: [u8; 8],
    pub revision: u32,
    pub header_size: u32,
    pub crc32_header: u32, // CRC32 of the header (from offset 0 to byte 91 (before the final
    // reserved field, with this crc32 field set to zero))
    pub _reserved1: u32, // must be zero
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub starting_lba_partition_entries: u64, // usually LBA 2 for compatability
    pub num_partition_entries: u32,
    pub size_of_partition_entry: u32,
    pub crc32_partition_array: u32, // CRC32 of the partition array (in little-endian)
    pub reserved: [u8; 420],        // to make the header 512 bytes
}

impl GPTHeader {
    pub fn read(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mbr = MBR::read_from_file(&mut file)?;
        // mbr partitions[0] should be GPT protective partition
        assert_eq!(
            mbr.partition_entries[0].partition_type,
            PartitionType::GPTProtective
        );
        // Seek to LBA 1 where GPT header is located
        // file is probably is already at this position after reading MBR
        // but seek anyway to be sure
        file.seek(std::io::SeekFrom::Start(512))?;

        let mut buffer = [0u8; size_of::<GPTHeader>()];
        file.read_exact(&mut buffer)?;
        // Safety: buffer is exactly the size of GPTHeader
        let header: GPTHeader = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
        header.assert_valid();
        Ok(header)
    }

    pub fn assert_valid(&self) {
        // Check signature
        assert!(
            self.signature == SIG_LITTLE_ENDIAN || self.signature == SIG_BIG_ENDIAN,
            "Invalid GPT signature: expected {:?} or {:?}, found {:?}",
            SIG_LITTLE_ENDIAN,
            SIG_BIG_ENDIAN,
            self.signature,
        );

        // Check revision
        assert_eq!(
            self.revision, 0x00010000,
            "Unsupported GPT revision: expected 0x00010000, found 0x{:08X}",
            self.revision
        );

        // Check header size
        assert_eq!(
            self.header_size, 92,
            "Invalid GPT header size: expected 92, found {}",
            self.header_size
        );

        // Check reserved field
        assert_eq!(
            self._reserved1, 0,
            "Invalid GPT reserved field: expected 0, found {}",
            self._reserved1
        );

        // check final reserved field
        let zeroed = [0u8; 420];
        assert_eq!(
            self.reserved, zeroed,
            "Invalid GPT reserved field: expected all zeros"
        );
    }
}

impl Debug for GPTHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GPTHeader")
            .field("revision", &format_args!("0x{:08X}", self.revision))
            .field("header_size", &self.header_size)
            .field("crc32_header", &format_args!("0x{:08X}", self.crc32_header))
            .field("current_lba", &self.current_lba)
            .field("backup_lba", &self.backup_lba)
            .field("first_usable_lba", &self.first_usable_lba)
            .field("last_usable_lba", &self.last_usable_lba)
            .field("disk_guid", &format_args!("{}", fmt_guid(&self.disk_guid)))
            .field(
                "starting_lba_partition_entries",
                &self.starting_lba_partition_entries,
            )
            .field("num_partition_entries", &self.num_partition_entries)
            .field("size_of_partition_entry", &self.size_of_partition_entry)
            .field(
                "crc32_partition_array",
                &format_args!("0x{:08X}", self.crc32_partition_array),
            )
            .field("reserved", &format_args!("[...420 bytes of zeros...]"))
            .finish()
    }
}

/// fmt_guid formats a 16-byte GUID into the standard string representation
fn fmt_guid(guid: &[u8; 16]) -> String {
    let mut out = String::with_capacity(36);
    for i in 0..16 {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            out.push('-');
        }
        out.push_str(&format!("{:02X}", guid[i]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_guid() {
        let cases = vec![
            ([0u8; 16], "00000000-0000-0000-0000-000000000000"),
            ([170u8; 16], "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA"),
            (
                [
                    0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55,
                    0x66, 0x77, 0x88,
                ],
                "12345678-9ABC-DEF0-1122-334455667788",
            ),
        ];
        for (case, expected) in cases {
            let formatted = fmt_guid(&case);
            assert_eq!(formatted, expected);
        }
    }
}
