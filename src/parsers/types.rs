// Types that all file parsers must use to extract data from their files.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{opt, value},
    multi::many1,
    IResult, Parser,
};
use std::{
    collections::HashMap,
    fmt::{Binary, Debug, Display, Formatter, LowerHex, Octal, Result, UpperHex},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    V0,
    V1,
    VX,
    VZ,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::V0 => write!(f, "0"),
            Value::V1 => write!(f, "1"),
            Value::VX => write!(f, "X"),
            Value::VZ => write!(f, "Z"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum WaveValue {
    Binary(Value),
    // String is always hexadecimal format internally
    Bus(String),
}

impl WaveValue {
    // Parse a single 4-state logic value
    fn parse_logic(input: &str) -> IResult<&str, Value> {
        alt((
            value(Value::V0, char('0')),
            value(Value::V1, char('1')),
            value(Value::VX, one_of("xX")),
            value(Value::VZ, one_of("zZ")),
        ))
        .parse(input)
    }

    // Parse a string into a vector of Value with a specific radix
    fn parse_string_to_vector_of_values(input: &str, radix: u32) -> IResult<&str, Vec<Value>> {
        // Handle the optional prefix based on radix
        let (input, _) = match radix {
            2 => opt(alt((tag("0b"), tag("0B")))).parse(input)?,
            8 => opt(alt((tag("0o"), tag("0O")))).parse(input)?,
            10 => opt(alt((tag("0d"), tag("0D")))).parse(input)?,
            16 => opt(alt((tag("0x"), tag("0X")))).parse(input)?,
            _ => (input, None),
        };

        // Parse digits based on radix
        let (input, values) = match radix {
            2 => many1(Self::parse_logic).parse(input)?,
            8 => {
                let (input, digits) = many1(one_of("01234567xXzZ")).parse(input)?;
                let values = digits
                    .into_iter()
                    .flat_map(|c| match c {
                        '0' => vec![Value::V0, Value::V0, Value::V0],
                        '1' => vec![Value::V0, Value::V0, Value::V1],
                        '2' => vec![Value::V0, Value::V1, Value::V0],
                        '3' => vec![Value::V0, Value::V1, Value::V1],
                        '4' => vec![Value::V1, Value::V0, Value::V0],
                        '5' => vec![Value::V1, Value::V0, Value::V1],
                        '6' => vec![Value::V1, Value::V1, Value::V0],
                        '7' => vec![Value::V1, Value::V1, Value::V1],
                        'x' | 'X' => vec![Value::VX, Value::VX, Value::VX],
                        'z' | 'Z' => vec![Value::VZ, Value::VZ, Value::VZ],
                        _ => vec![], // This should never happen
                    })
                    .collect();
                (input, values)
            }
            10 => {
                let (input, digits) = many1(one_of("0123456789xXzZ")).parse(input)?;
                let mut values = Vec::new();
                // Convert each decimal digit to its 4-bit binary representation
                for c in digits {
                    match c {
                        '0' => values.extend(vec![Value::V0, Value::V0, Value::V0, Value::V0]),
                        '1' => values.extend(vec![Value::V0, Value::V0, Value::V0, Value::V1]),
                        '2' => values.extend(vec![Value::V0, Value::V0, Value::V1, Value::V0]),
                        '3' => values.extend(vec![Value::V0, Value::V0, Value::V1, Value::V1]),
                        '4' => values.extend(vec![Value::V0, Value::V1, Value::V0, Value::V0]),
                        '5' => values.extend(vec![Value::V0, Value::V1, Value::V0, Value::V1]),
                        '6' => values.extend(vec![Value::V0, Value::V1, Value::V1, Value::V0]),
                        '7' => values.extend(vec![Value::V0, Value::V1, Value::V1, Value::V1]),
                        '8' => values.extend(vec![Value::V1, Value::V0, Value::V0, Value::V0]),
                        '9' => values.extend(vec![Value::V1, Value::V0, Value::V0, Value::V1]),
                        'x' | 'X' => {
                            values.extend(vec![Value::VX, Value::VX, Value::VX, Value::VX])
                        }
                        'z' | 'Z' => {
                            values.extend(vec![Value::VZ, Value::VZ, Value::VZ, Value::VZ])
                        }
                        _ => {} // This should never happen
                    }
                }
                (input, values)
            }
            16 => {
                let (input, digits) = many1(one_of("0123456789abcdefABCDEFxXzZ")).parse(input)?;
                let values = digits
                    .into_iter()
                    .flat_map(|c| match c.to_ascii_lowercase() {
                        '0' => vec![Value::V0, Value::V0, Value::V0, Value::V0],
                        '1' => vec![Value::V0, Value::V0, Value::V0, Value::V1],
                        '2' => vec![Value::V0, Value::V0, Value::V1, Value::V0],
                        '3' => vec![Value::V0, Value::V0, Value::V1, Value::V1],
                        '4' => vec![Value::V0, Value::V1, Value::V0, Value::V0],
                        '5' => vec![Value::V0, Value::V1, Value::V0, Value::V1],
                        '6' => vec![Value::V0, Value::V1, Value::V1, Value::V0],
                        '7' => vec![Value::V0, Value::V1, Value::V1, Value::V1],
                        '8' => vec![Value::V1, Value::V0, Value::V0, Value::V0],
                        '9' => vec![Value::V1, Value::V0, Value::V0, Value::V1],
                        'a' => vec![Value::V1, Value::V0, Value::V1, Value::V0],
                        'b' => vec![Value::V1, Value::V0, Value::V1, Value::V1],
                        'c' => vec![Value::V1, Value::V1, Value::V0, Value::V0],
                        'd' => vec![Value::V1, Value::V1, Value::V0, Value::V1],
                        'e' => vec![Value::V1, Value::V1, Value::V1, Value::V0],
                        'f' => vec![Value::V1, Value::V1, Value::V1, Value::V1],
                        'x' => vec![Value::VX, Value::VX, Value::VX, Value::VX],
                        'z' => vec![Value::VZ, Value::VZ, Value::VZ, Value::VZ],
                        _ => vec![], // This should never happen
                    })
                    .collect();
                (input, values)
            }
            _ => many1(Self::parse_logic).parse(input)?, // Default to binary
        };

        Ok((input, values))
    }

    pub fn values(&self, radix: u32) -> Option<Vec<Value>> {
        match self {
            WaveValue::Binary(v) => Some(vec![v.clone()]),
            WaveValue::Bus(s) => match Self::parse_string_to_vector_of_values(s, radix) {
                Ok((_, values)) => Some(values),
                Err(_) => None,
            },
        }
    }

    // Format a bus value with radix
    fn format_bus(&self, radix: u32, uppercase: bool, f: &mut Formatter<'_>) -> Result {
        match self {
            WaveValue::Binary(v) => write!(f, "{}", v),
            WaveValue::Bus(s) => {
                // Add prefix based on radix and format flags
                if f.alternate() {
                    match radix {
                        2 => write!(f, "0b")?,
                        8 => write!(f, "0o")?,
                        10 => write!(f, "0d")?,
                        16 => {
                            if uppercase {
                                write!(f, "0X")?
                            } else {
                                write!(f, "0x")?
                            }
                        }
                        _ => {}
                    }
                }

                // Use the appropriate parser based on radix
                match radix {
                    2 => {
                        write!(f, "{}", Self::format_string_as_binary(s, uppercase))
                    }
                    8 => {
                        write!(f, "{}", Self::format_string_as_octal(s, uppercase))
                    }
                    10 => {
                        write!(f, "{}", Self::format_string_as_decimal(s, uppercase))
                    }
                    16 => {
                        write!(f, "{}", Self::format_string_as_hex(s, uppercase))
                    }
                    _ => write!(f, "{}", s), // Default to original string for unsupported radix
                }
            }
        }
    }

    fn format_string_as_binary(s: &str, uppercase: bool) -> String {
        // If the string looks like it's already in binary format (only 0,1,x,z characters)
        // just return it without expanding
        if s.chars()
            .all(|c| c == '0' || c == '1' || c == 'x' || c == 'X' || c == 'z' || c == 'Z')
        {
            // Handle case conversion for non-digit characters
            if uppercase {
                return s
                    .chars()
                    .map(|c| {
                        if c == 'x' {
                            'X'
                        } else if c == 'z' {
                            'Z'
                        } else {
                            c
                        }
                    })
                    .collect();
            } else {
                return s
                    .chars()
                    .map(|c| {
                        if c == 'X' {
                            'x'
                        } else if c == 'Z' {
                            'z'
                        } else {
                            c
                        }
                    })
                    .collect();
            }
        }

        // Skip any hex prefix if present
        let s = s.trim_start_matches("0x").trim_start_matches("0X");

        let mut result = String::new();

        for c in s.chars() {
            match c {
                '0' => result.push_str("0000"),
                '1' => result.push_str("0001"),
                '2' => result.push_str("0010"),
                '3' => result.push_str("0011"),
                '4' => result.push_str("0100"),
                '5' => result.push_str("0101"),
                '6' => result.push_str("0110"),
                '7' => result.push_str("0111"),
                '8' => result.push_str("1000"),
                '9' => result.push_str("1001"),
                'a' | 'A' => result.push_str("1010"),
                'b' | 'B' => result.push_str("1011"),
                'c' | 'C' => result.push_str("1100"),
                'd' | 'D' => result.push_str("1101"),
                'e' | 'E' => result.push_str("1110"),
                'f' | 'F' => result.push_str("1111"),
                'x' | 'X' => {
                    let x = if uppercase { "X" } else { "x" };
                    result.push_str(x);
                }
                'z' | 'Z' => {
                    let z = if uppercase { "Z" } else { "z" };
                    result.push_str(z);
                }
                _ => {} // Skip invalid characters
            }
        }

        // Remove leading zeros, but keep at least one digit
        let result = result.trim_start_matches('0').to_string();
        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }

    fn format_string_as_decimal(s: &str, uppercase: bool) -> String {
        // Skip any prefixes if present
        let s = s
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .trim_start_matches("0b")
            .trim_start_matches("0B")
            .trim_start_matches("0o")
            .trim_start_matches("0O")
            .trim_start_matches("0d");

        // Try to convert to binary first, then to decimal
        // For simple cases without x/z values
        if !s.contains(|c| c == 'x' || c == 'X' || c == 'z' || c == 'Z') {
            // Try to parse as hex and convert to decimal
            if let Ok(num) = u64::from_str_radix(s, 16) {
                return num.to_string();
            }
        }

        // For values with x/z, just format the characters directly
        let result: String = s
            .chars()
            .map(|c| match c {
                '0'..='9' => c,
                'a'..='f' => c,
                'A'..='F' => c,
                'x' | 'X' => {
                    if uppercase {
                        'X'
                    } else {
                        'x'
                    }
                }
                'z' | 'Z' => {
                    if uppercase {
                        'Z'
                    } else {
                        'z'
                    }
                }
                _ => c,
            })
            .collect();

        result
    }

    fn format_string_as_octal(s: &str, uppercase: bool) -> String {
        // If the string contains x or z, handle it differently
        if s.contains(|c| c == 'x' || c == 'X' || c == 'z' || c == 'Z') {
            return s
                .chars()
                .map(|c| match c {
                    'x' => {
                        if uppercase {
                            'X'
                        } else {
                            'x'
                        }
                    }
                    'X' => {
                        if uppercase {
                            'X'
                        } else {
                            'x'
                        }
                    }
                    'z' => {
                        if uppercase {
                            'Z'
                        } else {
                            'z'
                        }
                    }
                    'Z' => {
                        if uppercase {
                            'Z'
                        } else {
                            'z'
                        }
                    }
                    _ => c,
                })
                .collect();
        }

        // Skip any hex prefix if present
        let s = s.trim_start_matches("0x").trim_start_matches("0X");

        // First convert to binary
        let binary = Self::format_string_as_binary(s, uppercase);

        // Then group by 3 bits from the right and convert to octal
        let mut result = String::new();
        let mut binary_padded = binary.clone();

        // Pad with leading zeros to make the length a multiple of 3
        while binary_padded.len() % 3 != 0 {
            binary_padded.insert(0, '0');
        }

        // Process each group of 3 bits
        for chunk in binary_padded.chars().collect::<Vec<_>>().chunks(3) {
            let chunk_str: String = chunk.iter().collect();
            match chunk_str.as_str() {
                "000" => result.push('0'),
                "001" => result.push('1'),
                "010" => result.push('2'),
                "011" => result.push('3'),
                "100" => result.push('4'),
                "101" => result.push('5'),
                "110" => result.push('6'),
                "111" => result.push('7'),
                _ if chunk_str.contains('x') || chunk_str.contains('X') => {
                    result.push(if uppercase { 'X' } else { 'x' });
                }
                _ if chunk_str.contains('z') || chunk_str.contains('Z') => {
                    result.push(if uppercase { 'Z' } else { 'z' });
                }
                _ => {} // Should never happen
            }
        }

        // Remove leading zeros, but keep at least one digit
        let result = result.trim_start_matches('0').to_string();
        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }

    // Format string as hex representation (preserve or change case as needed)
    fn format_string_as_hex(s: &str, uppercase: bool) -> String {
        // Skip any hex prefix if present
        let s = s.trim_start_matches("0x").trim_start_matches("0X");

        // Process each character and adjust case as needed
        let result: String = s
            .chars()
            .map(|c| match c {
                '0'..='9' => c,
                'a'..='f' => {
                    if uppercase {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    }
                }
                'A'..='F' => {
                    if uppercase {
                        c
                    } else {
                        c.to_ascii_lowercase()
                    }
                }
                'x' | 'X' => {
                    if uppercase {
                        'X'
                    } else {
                        'x'
                    }
                }
                'z' | 'Z' => {
                    if uppercase {
                        'Z'
                    } else {
                        'z'
                    }
                }
                _ => c, // Keep other characters as-is
            })
            .collect();

        // If the result is empty (unlikely), return "0"
        if result.is_empty() {
            "0".to_string()
        } else {
            result
        }
    }
}

impl Display for WaveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WaveValue::Binary(v) => write!(f, "{}", v),
            WaveValue::Bus(_) => self.format_bus(10, false, f),
        }
    }
}

impl Binary for WaveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.format_bus(2, false, f)
    }
}

impl Octal for WaveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.format_bus(8, false, f)
    }
}

impl LowerHex for WaveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.format_bus(16, false, f)
    }
}

impl UpperHex for WaveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.format_bus(16, true, f)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct WaveformData {
    pub signals: Vec<String>,
    pub values: HashMap<String, Vec<(u64, WaveValue)>>,
    pub max_time: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::V0), "0");
        assert_eq!(format!("{}", Value::V1), "1");
        assert_eq!(format!("{}", Value::VX), "X");
        assert_eq!(format!("{}", Value::VZ), "Z");
    }

    #[test]
    fn test_wave_value_display() {
        assert_eq!(format!("{}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{}", WaveValue::Binary(Value::V1)), "1");
        assert_eq!(format!("{}", WaveValue::Binary(Value::VX)), "X");
        assert_eq!(format!("{}", WaveValue::Binary(Value::VZ)), "Z");
    }

    #[test]
    fn test_binary_formatting() {
        // Test binary values
        assert_eq!(format!("{:b}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{:b}", WaveValue::Binary(Value::V1)), "1");

        // Test bus values with binary formatting
        let bus_a = WaveValue::Bus("a".to_string());
        let bus_f0 = WaveValue::Bus("f0".to_string());

        assert_eq!(format!("{:b}", bus_a), "1010");
        assert_eq!(format!("{:b}", bus_f0), "11110000");

        // Test with alternate form (#)
        assert_eq!(format!("{:#b}", bus_a), "0b1010");
        assert_eq!(format!("{:#b}", bus_f0), "0b11110000");
    }

    #[test]
    fn test_octal_formatting() {
        // Test binary values
        assert_eq!(format!("{:o}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{:o}", WaveValue::Binary(Value::V1)), "1");

        // Test bus values with octal formatting
        let bus_a = WaveValue::Bus("a".to_string());
        let bus_ff = WaveValue::Bus("ff".to_string());

        assert_eq!(format!("{:o}", bus_a), "12");
        assert_eq!(format!("{:o}", bus_ff), "377");

        // Test with alternate form (#)
        assert_eq!(format!("{:#o}", bus_a), "0o12");
        assert_eq!(format!("{:#o}", bus_ff), "0o377");
    }

    #[test]
    fn test_hex_formatting() {
        // Test binary values with lowercase hex
        assert_eq!(format!("{:x}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{:x}", WaveValue::Binary(Value::V1)), "1");

        // Test bus values with lowercase hex
        let bus_abcd = WaveValue::Bus("abcd".to_string());
        let bus_1234 = WaveValue::Bus("1234".to_string());

        assert_eq!(format!("{:x}", bus_abcd), "abcd");
        assert_eq!(format!("{:x}", bus_1234), "1234");

        // Test with alternate form (#)
        assert_eq!(format!("{:#x}", bus_abcd), "0xabcd");
        assert_eq!(format!("{:#x}", bus_1234), "0x1234");

        // Test binary values with uppercase hex
        assert_eq!(format!("{:X}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{:X}", WaveValue::Binary(Value::V1)), "1");

        // Test bus values with uppercase hex
        assert_eq!(format!("{:X}", bus_abcd), "ABCD");
        assert_eq!(format!("{:X}", bus_1234), "1234");

        // Test with alternate form (#)
        assert_eq!(format!("{:#X}", bus_abcd), "0XABCD");
        assert_eq!(format!("{:#X}", bus_1234), "0X1234");
    }

    #[test]
    fn test_hex_to_oct_conversion() {
        assert_eq!(format!("{:o}", WaveValue::Bus("7".to_string())), "7");
        assert_eq!(format!("{:o}", WaveValue::Bus("f0".to_string())), "360");
        assert_eq!(format!("{:o}", WaveValue::Bus("12".to_string())), "22");
        assert_eq!(format!("{:o}", WaveValue::Bus("x4".to_string())), "x4");
    }

    #[test]
    fn test_decimal_formatting() {
        assert_eq!(format!("{}", WaveValue::Binary(Value::V0)), "0");
        assert_eq!(format!("{}", WaveValue::Binary(Value::V1)), "1");

        assert_eq!(format!("{}", WaveValue::Bus("0".to_string())), "0");
        assert_eq!(format!("{}", WaveValue::Bus("9".to_string())), "9");
        assert_eq!(format!("{}", WaveValue::Bus("1".to_string())), "1");
        assert_eq!(format!("{}", WaveValue::Bus("a".to_string())), "10");
        assert_eq!(format!("{}", WaveValue::Bus("64".to_string())), "100");
        assert_eq!(format!("{}", WaveValue::Bus("3x".to_string())), "3x");
        assert_eq!(format!("{}", WaveValue::Bus("z6".to_string())), "z6");
    }
}
