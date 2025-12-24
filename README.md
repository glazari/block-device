# Block Device


This is an exploration of block devices on linux


Ideas of what to do
- Flash a usb drive with an empty partition
    - using /dev/sda
- 


# How to (with linux common tools)

Check partitions in a usb device?

```bash
# --------------------------------------------------
# using lsblk -f
# --------------------------------------------------
/bin/lsblk -f # gives some information about the block devices and the filesystems
# for some reason my lsblk is shadowed by the linuxbrew version (which shows less stuff)
NAME                  FSTYPE      FSVER    LABEL UUID                                   FSAVAIL FSUSE% MOUNTPOINTS
sda
└─sda1                vfat        FAT32          4936-CD7A                                14,3G     0% /media/glazari/4936-CD7A
nvme0n1
├─nvme0n1p1           vfat        FAT32          E9BE-3954                               504,8M     1% /boot/efi
├─nvme0n1p2           ext4        1.0            00000000-0000-0000-0000-000000000000      1,3G    13% /boot
└─nvme0n1p3           crypto_LUKS 2              00000000-0000-0000-0000-000000000000
  └─nvme0n1p3_crypt   LVM2_member LVM2 001       000000-0000-0000-0000-0000-0000-MLjthg
    ├─vgubuntu-root   ext4        1.0            00000000-0000-0000-0000-000000000000        1T    38% /
    └─vgubuntu-swap_1 swap        1              00000000-0000-0000-0000-000000000000                  [SWAP]


# --------------------------------------------------
# using file -s
# --------------------------------------------------

$ sudo file -s /dev/sda1

# /dev/sda1: DOS/MBR boot sector, code offset 0+2, OEM-ID "        ", 
# sectors/cluster 32, reserved sectors 30, Media descriptor 0xf8, 
# sectors/track 63, heads 255, hidden sectors 32, sectors 30031840 (volumes > 32 MB), FAT (32 bit), 
# sectors/FAT 7329, reserved 0x1, serial number 0x4936cd7a, unlabeled



```

Using fdisk

```bash
$ sudo /sbin/fdisk -l 
Disk /dev/sda: 14,32 GiB, 15376318464 bytes, 30031872 sectors
Disk model: Cruzer Blade
Units: sectors of 1 * 512 = 512 bytes
Sector size (logical/physical): 512 bytes / 512 bytes
I/O size (minimum/optimal): 512 bytes / 512 bytes
Disklabel type: dos
Disk identifier: 0x00000000

Device     Boot Start      End  Sectors  Size Id Type
/dev/sda1          32 30031871 30031840 14,3G  c W95 FAT32 (LBA)
```


Using parted


```bash
$ sudo parted -l 
Model: SanDisk Cruzer Blade (scsi)
Disk /dev/sda: 15,4GB
Sector size (logical/physical): 512B/512B
Partition Table: msdos
Disk Flags:

Number  Start   End     Size    Type     File system  Flags
 1      16,4kB  15,4GB  15,4GB  primary  fat32        lba

```


zero off a usb device (just the beggning)

```bash
# WARNING: deletes data
sudo dd if=/dev/zero of=/dev/sda bs=1M count=10 status=progress
# the whole file
sudo dd if=/dev/zero of=/dev/sda bs=4M status=progress
```


Re adding a partition table and file system
```bash
sudo parted /dev/sda mklabel gpt
sudo parted /dev/sda mkpart primary 0% 100%
sudo mkfs.ext4 /dev/sda1 
```

for fat32 (better use different partition table

```bash
sudo parted /dev/sda mklabel msdos
sudo parted /dev/sda mkpart primary fat32 1MiB 100%
sudo mkfs.vfat -F 32 /dev/sda1
```

Existing partition table options in parted
```
aix, amiga, bsd, dvh, gpt, loop, mac, msdos, pc98, sun
```

Existing filesystem options for mkfs
```
bfs cramfs ext2 ext3 ext4 f2fs fat minix msdos ntfs vfat
```
