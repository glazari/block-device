//! Master Boot Record (MBR)
//!
//! its the partition table from msdos.
//! Supports 4 primary partitions on a disk.
//!
//! # Using structure found in wikipedia
//! https://en.wikipedia.org/wiki/Master_boot_record
//!
//! There are several formats described there, using the
//! one they describe as classical generic MBR

use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::io::Result;
use std::mem::size_of;

// Invariants
const _: () = assert!(size_of::<MBR>() == 512, "MBR size must be 512 bytes");
const _: () = assert!(
    size_of::<PartitionEntry>() == 16,
    "PartitionEntry size must be 16 bytes"
);
const _: () = assert!(size_of::<CHS>() == 3, "CHS size must be 3 bytes");

/// Master Boot Record (MBR)
#[repr(C)]
#[derive(Clone)]
pub struct MBR {
    pub boot_code: [u8; 446],
    pub partition_entries: [PartitionEntry; 4],
    pub signature: [u8; 2],
}

#[repr(C)]
#[derive(Clone)]
pub struct PartitionEntry {
    pub boot_indicator: u8, // 0x80 = bootable, 0x00 = non-bootable
    pub starting_chs: CHS,
    pub partition_type: PartitionType,
    pub ending_chs: CHS,
    pub starting_lba: LBA,
    pub size_in_lba: LBA,
}

#[repr(C)]
/// Logical Block Addressing (LBA)
#[derive(Clone)]
pub struct LBA {
    pub lba: [u8; 4],
}

/// Cylinder-Head-Sector (CHS) Addressing
/// This is mostly obsolete,
/// most modern tools look only at LBA
/// in many cases CHS values are set to maximum values
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CHS {
    pub head: u8,
    pub sector: u8,
    pub cylinder: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // These are the types, even if we don't use them
pub enum PartitionType {
    Empty = 0x00,
    FAT12 = 0x01,
    XENIXRoot = 0x02,
    FAT16Small = 0x04,
    Extended = 0x05,
    FAT16Large = 0x06,
    NTFS = 0x07,
    FAT32 = 0x0B,
    FAT32LBA = 0x0C,
    FAT16LargeLBA = 0x0E,
    ExtendedLBA = 0x0F,
    LinuxSwap = 0x82,
    LinuxNative = 0x83,
    LinuxExtended = 0x85,
    WindowsRecoveryEnv = 0x27,
    HiddenNTFS = 0x17,

    // 0xEE is used for GPT protective partitions
    GPTProtective = 0xEE,

    // WARNING: There are many more partition types,
    // And we simply reintepret the u8 value as PartitionType
    // so if the value is not in this enum,
    // Rust will just treat it as random value
}

impl Debug for MBR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MBR")
            .field(
                "boot_code",
                &format_args!("[{} bytes]", self.boot_code.len()),
            )
            .field("partition_entries", &self.partition_entries)
            .field(
                "signature",
                &format_args!("0x{:02X}{:02X}", self.signature[1], self.signature[0]),
            )
            .finish()
    }
}
impl Debug for PartitionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let invalid = format!("Invalid Boot Indicator {}", self.boot_indicator);
        let bootable = match self.boot_indicator {
            0x80 => "Bootable",
            0x00 => "Not Bootable",
            _ => &invalid,
        };
        f.debug_tuple("PartitionEntry")
            .field(&format_args!("{:?} ({bootable})", self.partition_type))
            .field(&format_args!(
                "chs[{:?} -> {:?}]",
                self.starting_chs.val(),
                self.ending_chs.val()
            ))
            .field(&format_args!(
                "lba[{} -> {}] (size: {})",
                self.starting_lba.val(),
                self.starting_lba.val() + self.size_in_lba.val(),
                self.size_in_lba.val()
            ))
            .finish()
    }
}

impl Debug for LBA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LBA(0x{})", self.val())
    }
}

impl LBA {
    pub fn val(&self) -> u32 {
        u32::from_le_bytes(self.lba)
    }
}

impl Debug for CHS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cylinder, head, sector) = self.val();
        write!(f, "CHS({cylinder}, {head}, {sector})")
    }
}

impl CHS {
    pub fn val(&self) -> (u16, u8, u8) {
        // cylinder number is 10 bits: 2 bits from sector and 8 bits from cylinder
        let cylinder = ((self.sector as u16 & 0xC0) << 2) | self.cylinder as u16;
        // removing the 2 bits used for cylinder from sector
        let sector = self.sector & 0x3F;
        (cylinder, self.head, sector)
    }
    #[allow(dead_code)]
    pub fn max() -> Self {
        CHS {
            head: 0xFF,
            sector: 0x02,
            cylinder: 0xFF,
        }
    }
}

impl MBR {
    pub fn read(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; size_of::<MBR>()];
        file.read_exact(&mut buffer)?;
        // Safety: We are reading from a byte array of the correct size
        let mbr: MBR = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
        Ok(mbr)
    }
}
