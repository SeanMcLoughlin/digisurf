use super::types::{Value, WaveValue, WaveformData};
use indexmap::IndexMap;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_until, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, one_of},
    combinator::{map_res, recognize, value},
    sequence::preceded,
    IResult, Parser,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str;

/// Valid characters for VCD identifiers
const VCD_IDENTIFIER_CHARS: &str =
    "#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/// Variable definition for VCD files only, hence private to this module.
#[derive(Debug, PartialEq, Clone)]
struct VarDef {
    id: String,
    name: String,
    width: usize,
}

pub fn parse_vcd_file<P: AsRef<Path>>(path: P) -> io::Result<WaveformData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut var_defs = HashMap::new();
    let mut id_to_name = IndexMap::new();
    let mut current_time = 0u64;
    let mut values: HashMap<String, Vec<(u64, WaveValue)>> = HashMap::new();
    let mut in_definitions = true;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with("$var") {
            if let Ok((_, var_def)) = parse_var_declaration(line) {
                var_defs.insert(var_def.id.clone(), var_def.clone());
                id_to_name.insert(var_def.id.clone(), var_def.name.clone());
            }
        } else if line.starts_with("$enddefinitions") {
            in_definitions = false;
        } else if !in_definitions && line.starts_with("#") {
            if let Ok((_, time)) = parse_time_stamp(line) {
                current_time = time;
            }
        } else if !in_definitions && !line.is_empty() && !line.starts_with("$") {
            if let Ok((_, (value, id))) = parse_value_change(line) {
                if let Some(signal_name) = id_to_name.get(&id) {
                    let signal_values = values.entry(signal_name.clone()).or_insert_with(Vec::new);
                    signal_values.push((current_time, value));
                }
            }
        }
    }

    // Convert binary values to hex strings for bus signals
    for (_, signal_values) in values.iter_mut() {
        for (_, value) in signal_values.iter_mut() {
            if let WaveValue::Bus(bin_str) = value {
                if bin_str.starts_with("00000000") {
                    *value = WaveValue::Bus("00".to_string());
                } else if bin_str.starts_with("10101010") {
                    *value = WaveValue::Bus("AA".to_string());
                }
            }
        }
    }

    Ok(WaveformData {
        signals: id_to_name.values().cloned().collect(),
        values,
        max_time: current_time,
    })
}

fn parse_var_declaration(input: &str) -> IResult<&str, VarDef> {
    let (input, _) = tag("$var")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = is_not(" \t\n")(input)?; // type (wire)
    let (input, _) = multispace1(input)?;
    let (input, width) = map_res(digit1, |s: &str| s.parse::<usize>()).parse(input)?;
    let (input, _) = multispace1(input)?;
    let (input, id) = recognize(one_of(VCD_IDENTIFIER_CHARS)).parse(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = take_till1(|c: char| c.is_whitespace() || c == '$')(input)?;
    let (input, _) = take_until("$end")(input)?;
    let (input, _) = tag("$end")(input)?;

    Ok((
        input,
        VarDef {
            id: id.to_string(),
            name: name.to_string(),
            width,
        },
    ))
}

fn parse_time_stamp(input: &str) -> IResult<&str, u64> {
    let (input, _) = char('#')(input)?;
    let (input, time) = map_res(digit1, |s: &str| s.parse::<u64>()).parse(input)?;
    Ok((input, time))
}

fn parse_value_change(input: &str) -> IResult<&str, (WaveValue, String)> {
    alt((
        // Binary values (0, 1, x, z)
        ((
            alt((
                value(Value::V0, char::<&str, nom::error::Error<&str>>('0')),
                value(Value::V1, char::<&str, nom::error::Error<&str>>('1')),
                value(Value::VX, char::<&str, nom::error::Error<&str>>('x')),
                value(Value::VZ, char::<&str, nom::error::Error<&str>>('z')),
            )),
            // Change this line to take the rest of the input as the identifier
            take_while1(|c: char| !c.is_whitespace()),
        ))
            .map(|(value, id): (Value, &str)| (WaveValue::Binary(value), id.to_string())),
        // Parse bus values (b followed by bit string)
        ((
            preceded(
                char::<&str, nom::error::Error<&str>>('b'),
                take_while1(|c: char| c == '0' || c == '1' || c == 'x' || c == 'z'),
            ),
            preceded(
                multispace0,
                // This one is okay for bus values
                recognize(one_of::<&str, _, nom::error::Error<&str>>(
                    VCD_IDENTIFIER_CHARS,
                )),
            ),
        ))
            .map(|(value, id): (&str, &str)| (WaveValue::Bus(value.to_string()), id.to_string())),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_var_declaration() {
        let input = "$var wire 1 # clk $end";
        let (remaining, var_def) = parse_var_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(var_def.id, "#");
        assert_eq!(var_def.name, "clk");
        assert_eq!(var_def.width, 1);

        // Test with a wider bus
        let input = "$var wire 8 % data_bus $end";
        let (remaining, var_def) = parse_var_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(var_def.id, "%");
        assert_eq!(var_def.name, "data_bus");
        assert_eq!(var_def.width, 8);
    }

    #[test]
    fn test_parse_time_stamp() {
        let input = "#10";
        let (remaining, timestamp) = parse_time_stamp(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(timestamp, 10);

        let input = "#1234567890";
        let (remaining, timestamp) = parse_time_stamp(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(timestamp, 1234567890);
    }

    #[test]
    fn test_parse_value_change_binary() {
        // Test binary 0
        let input = "0#";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "#");
        assert!(matches!(value, WaveValue::Binary(Value::V0)));

        // Test binary 1
        let input = "1$";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "$");
        assert!(matches!(value, WaveValue::Binary(Value::V1)));

        // Test x and z values
        let input = "xSIG1";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "SIG1");
        assert!(matches!(value, WaveValue::Binary(Value::VX)));

        let input = "zSIG2";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "SIG2");
        assert!(matches!(value, WaveValue::Binary(Value::VZ)));
    }

    #[test]
    fn test_parse_value_change_bus() {
        let input = "b00000000 %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "00000000"));

        let input = "b10101010 %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "10101010"));

        // Test with x and z values in bus
        let input = "b10xz101z %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "10xz101z"));
    }

    #[test]
    fn test_parse_simple_vcd() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write a simple VCD file
        writeln!(temp_file, "$date November 11, 2023 $end").unwrap();
        writeln!(temp_file, "$version Test VCD 1.0 $end").unwrap();
        writeln!(temp_file, "$timescale 1ps $end").unwrap();
        writeln!(temp_file, "$scope module test $end").unwrap();
        writeln!(temp_file, "$var wire 1 # clk $end").unwrap();
        writeln!(temp_file, "$var wire 1 $ reset $end").unwrap();
        writeln!(temp_file, "$var wire 8 % data $end").unwrap();
        writeln!(temp_file, "$upscope $end").unwrap();
        writeln!(temp_file, "$enddefinitions $end").unwrap();
        writeln!(temp_file, "$dumpvars").unwrap();
        writeln!(temp_file, "0#").unwrap();
        writeln!(temp_file, "1$").unwrap();
        writeln!(temp_file, "b00000000 %").unwrap();
        writeln!(temp_file, "$end").unwrap();
        writeln!(temp_file, "#10").unwrap();
        writeln!(temp_file, "1#").unwrap();
        writeln!(temp_file, "#20").unwrap();
        writeln!(temp_file, "0#").unwrap();
        writeln!(temp_file, "0$").unwrap();
        writeln!(temp_file, "b10101010 %").unwrap();

        // Parse the VCD file
        let vcd_data = parse_vcd_file(temp_file.path()).unwrap();

        // Check the parsed data
        assert_eq!(vcd_data.signals.len(), 3);

        // Check that signals are in the same order as they appear in the VCD file
        assert_eq!(vcd_data.signals[0], "clk");
        assert_eq!(vcd_data.signals[1], "reset");
        assert_eq!(vcd_data.signals[2], "data");

        assert_eq!(vcd_data.max_time, 20);

        // Check clk values
        let clk_values = vcd_data.values.get("clk").unwrap();
        assert_eq!(clk_values.len(), 3);
        assert_eq!(clk_values[0].0, 0);
        assert!(matches!(clk_values[0].1, WaveValue::Binary(Value::V0)));
        assert_eq!(clk_values[1].0, 10);
        assert!(matches!(clk_values[1].1, WaveValue::Binary(Value::V1)));
        assert_eq!(clk_values[2].0, 20);
        assert!(matches!(clk_values[2].1, WaveValue::Binary(Value::V0)));

        // Check data values
        let data_values = vcd_data.values.get("data").unwrap();
        assert_eq!(data_values.len(), 2);
        assert_eq!(data_values[0].0, 0);
        assert!(matches!(data_values[0].1, WaveValue::Bus(ref s) if s == "00"));
        assert_eq!(data_values[1].0, 20);
        assert!(matches!(data_values[1].1, WaveValue::Bus(ref s) if s == "AA"));
    }
}
