pub fn format_bytes(data: &[u8]) -> String {
    let mut formatted = String::from("b'");

    for &byte in data {
        if (32..=126).contains(&byte) {
            formatted.push(char::from(byte));
        } else {
            formatted.push_str(&format!("\\x{byte:02x}"));
        }
    }

    formatted.push('\'');
    formatted
}

#[cfg(test)]
mod tests {
    use super::format_bytes;

    #[test]
    fn formats_printable_ascii_as_plain_text() {
        assert_eq!(format_bytes(b"hello"), "b'hello'");
    }

    #[test]
    fn formats_binary_bytes_with_hex_escapes() {
        assert_eq!(format_bytes(&[0x00, b'A', 0xff]), "b'\\x00A\\xff'");
    }
}
