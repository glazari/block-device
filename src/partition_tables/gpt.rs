//! GUID Partition Table (GPT)
//!
//! It is part of the Unified Extensible Firmware Interface (UEFI) standard.
//!
//! Supports more than 4 primary partitions on a disk.
//! Supports disks larger than 2 TB.

use std::{
    fmt::{Debug, Write},
    fs::File,
    io::{Read, Seek},
};

use crate::{
    partition_tables::msdos::{MBR, PartitionType},
    utils::size_to_human_readable,
};

// Invariants
const _: () = assert!(
    std::mem::size_of::<GPTHeader>() == SECTOR_SIZE as usize,
    "GPTHeader size must be 512 bytes"
);
const _: () = assert!(
    std::mem::size_of::<GPTPartitionEntry>() == 128,
    "GPTPartitionEntry size must be 128 bytes"
);

const SIGNATURE: &str = "EFI PART";
const SECTOR_SIZE: u64 = 512;

#[repr(C)]
#[derive(Clone)]
pub struct GPT {
    pub header: GPTHeader,
    pub partition_entries: Vec<GPTPartitionEntry>,
}

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
    pub disk_guid: Guid,
    pub starting_lba_partition_entries: u64, // usually LBA 2 for compatability
    pub num_partition_entries: u32,
    pub size_of_partition_entry: u32,
    pub crc32_partition_array: u32, // CRC32 of the partition array (in little-endian)
    pub reserved: [u8; 420],        // to make the header SECTOR_SIZE bytes
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct Guid([u8; 16]);

#[repr(C)]
#[derive(Clone)]
pub struct GPTPartitionEntry {
    pub partition_type: Guid,
    pub partition_id: Guid,
    pub first_lba: u64,
    pub last_lba: u64,        // inclusive, usually odd
    pub attribute_flags: u64, // 60 denotes read only
    pub name: [u16; 36],      // utf-16 code points of the name
}

impl GPT {
    pub fn read(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let header = GPTHeader::read_from_file(&mut file)?;
        let num_entries = header.num_partition_entries as usize;
        let mut entries: Vec<GPTPartitionEntry> = Vec::with_capacity(num_entries);

        // Seek to starting_lba_partition_entries
        let lba = header.starting_lba_partition_entries;
        file.seek(std::io::SeekFrom::Start(lba * SECTOR_SIZE))?;

        // Safety: We have allocated enough space for num_entries
        // And GPTPartitionEntry is #[repr(C)] so no padding issues
        unsafe {
            entries.set_len(num_entries);
            let buffer = std::slice::from_raw_parts_mut(
                entries.as_mut_ptr() as *mut u8,
                num_entries * std::mem::size_of::<GPTPartitionEntry>(),
            );
            file.read_exact(buffer)?;
        }

        Ok(GPT {
            header,
            partition_entries: entries,
        })
    }
}

impl GPTHeader {
    pub fn read(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        Self::read_from_file(&mut file)
    }
    pub fn read_from_file(file: &mut File) -> std::io::Result<Self> {
        let mbr = MBR::read_from_file(file)?;
        // mbr partitions[0] should be GPT protective partition
        assert_eq!(
            mbr.partition_entries[0].partition_type,
            PartitionType::GPTProtective
        );
        // Seek to LBA 1 where GPT header is located
        // file is probably is already at this position after reading MBR
        // but seek anyway to be sure
        file.seek(std::io::SeekFrom::Start(SECTOR_SIZE))?;

        let mut buffer = [0u8; size_of::<GPTHeader>()];
        file.read_exact(&mut buffer)?;
        // Safety: buffer is exactly the size of GPTHeader
        let header: GPTHeader = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
        header.assert_valid();
        Ok(header)
    }

    pub fn assert_valid(&self) {
        // Check signature
        let sig_bytes = SIGNATURE.as_bytes();
        assert!(
            self.signature.as_slice() == sig_bytes,
            "Invalid GPT signature: expected {:X?} , found {:X?}",
            sig_bytes,
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

        // Check partition_entry size
        assert_eq!(
            self.size_of_partition_entry, 128,
            "Invalid GPT partition entry size: expected 128, found {}",
            self.size_of_partition_entry
        );

        // check final reserved field
        let zeroed = [0u8; 420];
        assert_eq!(
            self.reserved, zeroed,
            "Invalid GPT reserved field: expected all zeros"
        );

        // TODO: Check crc32 fields
        // - crc32_header
        // - crc32_partition_array
    }
}

impl Debug for GPT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut not_emtpy_partitions = Vec::with_capacity(self.partition_entries.len());
        for entry in &self.partition_entries {
            if entry.partition_type != Guid([0u8; 16]) {
                not_emtpy_partitions.push(entry);
            }
        }
        f.debug_struct("GPT")
            .field("header", &self.header)
            .field("partition_entries", &not_emtpy_partitions)
            .finish()
    }
}

impl Debug for GPTPartitionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Decode UTF-16 name
        let end_name = self
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.name.len());

        let name = String::from_utf16(&self.name[..end_name])
            .unwrap_or_else(|_| "<invalid utf-16>".to_string());

        if name.is_empty() && self.partition_type == Guid([0u8; 16]) {
            return write!(f, "GPTPartitionEntry <unused>");
        }

        // TODO: show more compact view, and give some types names
        let size = (self.last_lba - self.first_lba + 1) * SECTOR_SIZE;
        f.debug_struct("GPTPartitionEntry")
            .field("name", &name)
            .field("partition_type", &self.partition_type)
            .field("partition_id", &self.partition_id)
            .field(
                "lba",
                &format_args!(
                    "{} -> {} ({})",
                    &self.first_lba,
                    &self.last_lba,
                    size_to_human_readable(size)
                ),
            )
            .field(
                "attribute_flags",
                &format_args!("0x{:016X}", self.attribute_flags),
            )
            .finish()
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
            .field("disk_guid", &self.disk_guid)
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
impl Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guid = self.0;
        let data1 = u32::from_le_bytes([guid[0], guid[1], guid[2], guid[3]]);
        let data2 = u16::from_le_bytes([guid[4], guid[5]]);
        let data3 = u16::from_le_bytes([guid[6], guid[7]]);
        // big endian portion
        let data4 = u16::from_be_bytes([guid[8], guid[9]]);
        let data5 = &guid[10..16];
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:04X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            data1, data2, data3, data4, data5[0], data5[1], data5[2], data5[3], data5[4], data5[5]
        )?;

        Ok(())
    }
}

pub const EFI_SYSTEM_PARTITION: Guid = guid_from_str("C12A7328-F81F-11D2-BA4B-00A0C93EC93B");
pub const LINUX_FILESYSTEM_DATA: Guid = guid_from_str("0FC63DAF-8483-4772-8E79-3D69D8477DE4");
pub const LINUX_SWAP: Guid = guid_from_str("0657FD6D-A4AB-43C4-84E5-0933C84B4F4F");
pub const LINUX_LVM: Guid = guid_from_str("E6D6D379-F507-44C2-A23C-238F2A3DF928");
pub const WINDOWS_BASIC_DATA: Guid = guid_from_str("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7");
pub const WINDOWS_RECOVERY_ENVIRONMENT: Guid =
    guid_from_str("DE94BBA4-06D1-4D40-A16A-BFD50179D6AC");

pub const fn guid_from_str(s: &str) -> Guid {
    const fn hex(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => 10 + (b - b'a'),
            b'A'..=b'F' => 10 + (b - b'A'),
            _ => panic!("invalid hex"),
        }
    }

    const fn byte(bytes: &[u8], i: usize) -> u8 {
        (hex(bytes[i]) << 4) | hex(bytes[i + 1])
    }

    let bytes = s.as_bytes();

    // "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" (36 chars)
    assert!(bytes.len() == 36);
    assert!(bytes[8] == b'-');
    assert!(bytes[13] == b'-');
    assert!(bytes[18] == b'-');
    assert!(bytes[23] == b'-');

    let mut g = [0u8; 16];

    // data1 (u32, little-endian)
    g[0] = byte(bytes, 6); // xx33
    g[1] = byte(bytes, 4); // xx22
    g[2] = byte(bytes, 2); // xx11
    g[3] = byte(bytes, 0); // xx00

    // data2 (u16, little-endian)
    g[4] = byte(bytes, 11); // xx55
    g[5] = byte(bytes, 9); // xx44

    // data3 (u16, little-endian)
    g[6] = byte(bytes, 16); // xx77
    g[7] = byte(bytes, 14); // xx66

    // data4 (u16, big-endian)
    g[8] = byte(bytes, 19); // xx88
    g[9] = byte(bytes, 21); // xx99

    // data5 (6 bytes, as-is)
    g[10] = byte(bytes, 24);
    g[11] = byte(bytes, 26);
    g[12] = byte(bytes, 28);
    g[13] = byte(bytes, 30);
    g[14] = byte(bytes, 32);
    g[15] = byte(bytes, 34);

    Guid(g)
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
                // mixed endienness test
                [
                    0x78, 0x56, 0x34, 0x12, // first little-endian part
                    0xBC, 0x9A, // second little-endian part
                    0xF0, 0xDE, // third little-endian part
                    // Starting here big-endian
                    0x11, 0x22, // first big-endian part
                    0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // second big-endian part
                ],
                "12345678-9ABC-DEF0-1122-334455667788",
            ),
        ];
        for (case, expected) in cases {
            let guid = Guid(case);
            let formatted = format!("{guid:?}");
            assert_eq!(formatted, expected);

            let extracted_guid = guid_from_str(expected);
            assert_eq!(extracted_guid, guid);
        }
    }
}
