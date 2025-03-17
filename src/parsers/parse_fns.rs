use nom::IResult;

pub fn parse_binary_to_hex(bin_str: &str) -> IResult<&str, String> {
    if bin_str.is_empty() {
        return Ok(("", String::new()));
    }

    // Check if the string contains only valid binary digits
    if !bin_str.chars().all(|c| c == '0' || c == '1') {
        return Err(nom::Err::Error(nom::error::Error::new(
            bin_str,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Pad with leading zeros to make length a multiple of 4
    let padding = (4 - (bin_str.len() % 4)) % 4;
    let padded = "0".repeat(padding) + bin_str;

    // Convert chunks of 4 bits to hex characters
    let hex_result = padded
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| {
            let value = chunk
                .iter()
                .fold(0u8, |acc, &bit| (acc << 1) | if bit == '1' { 1 } else { 0 });
            char::from_digit(value as u32, 16)
                .unwrap()
                .to_ascii_uppercase()
        })
        .collect();

    Ok(("", hex_result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_binary_to_hex() {
        // Empty input to empty input
        let input = "";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "");

        // Pad zeroes one nibble
        let input = "0";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "0");

        // No need to pad zeroes, one nibble
        let input = "0000";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "0");

        // Pad zeroes two nibbles
        let input = "00000";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "00");

        // No need to pad zeroes, two nibbles
        let input = "00000000";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "00");

        // Pad zeroes one nibble, keep 1 as the LSB
        let input = "1";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "1");

        // Pad zeroes one nibble, keep 111 as the LSBs
        let input = "111";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "7");

        // Pad zeroes two nibbles, keep 7F
        let input = "1111111";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "7F");

        // No need to pad zeroes, two nibbles, keep FF
        let input = "11111111";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "FF");

        // Something other than 0 or F
        let input = "10101010";
        let (remaining, hex_str) = parse_binary_to_hex(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str, "AA");

        // Test with a large binary input (512 bits)
        let mut large_input = String::new();
        for _ in 0..512 {
            large_input.push('1');
        }
        let (remaining, hex_str) = parse_binary_to_hex(&large_input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(hex_str.len(), 128); // 512 bits = 128 hex chars
        assert!(hex_str.chars().all(|c| c == 'F')); // All 1s should be all Fs

        // Non-binary chars == Error
        let input = "101a101";
        assert!(parse_binary_to_hex(input).is_err());

        // Non-binary chars == Error
        let input = "101x101";
        assert!(parse_binary_to_hex(input).is_err());
    }
}
