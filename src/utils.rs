const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;

pub fn size_to_human_readable(size_in_bytes: u64) -> String {
    match size_in_bytes {
        s if s >= TB => format!("{:.2} TB", s as f64 / TB as f64),
        s if s >= GB => format!("{:.2} GB", s as f64 / GB as f64),
        s if s >= MB => format!("{:.2} MB", s as f64 / MB as f64),
        s if s >= KB => format!("{:.2} KB", s as f64 / KB as f64),
        s => format!("{} B", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_to_human_readable() {
        let cases = vec![
            (500, "500 B"),
            (2048, "2.00 KB"),
            (5 * MB, "5.00 MB"),
            (3 * GB + 512 * MB, "3.50 GB"),
            (2 * TB + 256 * GB, "2.25 TB"),
            (1 * TB - 1 * GB, "1023.00 GB"),
            (1 * GB - 1 * MB, "1023.00 MB"),
            (1 * MB - 1 * KB, "1023.00 KB"),
            (1 * KB - 1, "1023 B"),
        ];

        for (input, expected) in cases {
            assert_eq!(size_to_human_readable(input), expected);
        }
    }
}
