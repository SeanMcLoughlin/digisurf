use super::parse_fns::*;
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
    "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";

/// Variable definition for VCD files only, hence private to this module.
#[derive(Debug, PartialEq, Clone)]
struct VarDef {
    id: String,
    name: String,
    width: usize,
    var_type: String,
}

pub fn parse_vcd_file<P: AsRef<Path>>(path: P) -> io::Result<WaveformData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut var_defs = HashMap::new();
    let mut id_to_name = IndexMap::new();
    let mut current_time = 0u64;
    let mut values: HashMap<String, Vec<(u64, WaveValue)>> = HashMap::new();
    let mut in_definitions = true;
    let mut in_dumpvars = false;
    let mut current_scope = Vec::<String>::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with("$var") {
            if let Ok((_, var_def)) = parse_var_declaration(line) {
                // Combine full hierarchical name using the current scope
                let mut full_name = String::new();
                for scope in &current_scope {
                    full_name.push_str(scope);
                    full_name.push('.');
                }
                full_name.push_str(&var_def.name);

                var_defs.insert(var_def.id.clone(), var_def.clone());
                id_to_name.insert(var_def.id.clone(), full_name);
            }
        } else if line.starts_with("$scope") {
            if let Ok((_, scope_name)) = parse_scope_declaration(line) {
                current_scope.push(scope_name);
            }
        } else if line.starts_with("$upscope") {
            if !current_scope.is_empty() {
                current_scope.pop();
            }
        } else if line.starts_with("$enddefinitions") {
            in_definitions = false;
        } else if line.starts_with("$dumpvars") {
            in_dumpvars = true;
        } else if line.starts_with("$end") && in_dumpvars {
            in_dumpvars = false;
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
                // Only convert if the value looks like a binary string (all 0s and 1s)
                if bin_str.chars().all(|c| c == '0' || c == '1') {
                    if let Ok((_, hex_str)) = parse_binary_to_hex(bin_str) {
                        *value = WaveValue::Bus(hex_str);
                    }
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

fn parse_scope_declaration(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("$scope")(input)?;
    let (input, _) = multispace1(input)?;

    // Scope type (module, task, function, begin, fork) - we don't actually need this for display
    let (input, _) = is_not(" \t\n")(input)?;
    let (input, _) = multispace1(input)?;

    // Scope identifier
    let (input, name) = take_till1(|c: char| c.is_whitespace() || c == '$')(input)?;

    // End of declaration
    let (input, _) = take_until("$end")(input)?;
    let (input, _) = tag("$end")(input)?;

    Ok((input, name.to_string()))
}

fn parse_var_declaration(input: &str) -> IResult<&str, VarDef> {
    let (input, _) = tag("$var")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, var_type) = is_not(" \t\n")(input)?; // type (wire, reg, etc.)
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
            var_type: var_type.to_string(),
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
                value(Value::VX, one_of::<&str, _, nom::error::Error<&str>>("xX")),
                value(Value::VZ, one_of::<&str, _, nom::error::Error<&str>>("zZ")),
            )),
            // The identifier follows the value with no whitespace
            take_while1(|c: char| VCD_IDENTIFIER_CHARS.contains(c)),
        ))
            .map(|(value, id): (Value, &str)| (WaveValue::Binary(value), id.to_string())),
        // Parse bus values (b followed by bit string)
        ((
            preceded(
                one_of::<&str, _, nom::error::Error<&str>>("bB"), // Support both 'b' and 'B' prefixes
                take_while1(|c: char| "01xXzZ".contains(c)),
            ),
            preceded(
                multispace0,
                // Take the identifier (one or more valid identifier chars)
                take_while1(|c: char| VCD_IDENTIFIER_CHARS.contains(c)),
            ),
        ))
            .map(|(value, id): (&str, &str)| (WaveValue::Bus(value.to_string()), id.to_string())),
        // Support for real values (r followed by a real number)
        ((
            preceded(
                one_of::<&str, _, nom::error::Error<&str>>("rR"),
                take_while1(|c: char| "0123456789.eE+-".contains(c)),
            ),
            preceded(
                multispace0,
                take_while1(|c: char| VCD_IDENTIFIER_CHARS.contains(c)),
            ),
        ))
            .map(|(value, id): (&str, &str)| {
                // FIXME: Real values are simply placed into a bus format right now. There is no
                // intention for this app to support mixed signal waveforms, and I don't think that
                // a TUI would be a good UI for viewing non-binary signals. However, the parser
                // should still be able to handle these values. Need to review this to determine if
                // a new enum variant should be added for real values.
                (WaveValue::Bus(format!("r{}", value)), id.to_string())
            }),
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
        assert_eq!(var_def.var_type, "wire");

        // Test with a wider bus
        let input = "$var wire 8 % data_bus $end";
        let (remaining, var_def) = parse_var_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(var_def.id, "%");
        assert_eq!(var_def.name, "data_bus");
        assert_eq!(var_def.width, 8);
        assert_eq!(var_def.var_type, "wire");

        // Test with a different var type
        let input = "$var reg 32 @ address $end";
        let (remaining, var_def) = parse_var_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(var_def.id, "@");
        assert_eq!(var_def.name, "address");
        assert_eq!(var_def.width, 32);
        assert_eq!(var_def.var_type, "reg");
    }

    #[test]
    fn test_parse_scope_declaration() {
        let input = "$scope module top $end";
        let (remaining, scope_name) = parse_scope_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(scope_name, "top");

        let input = "$scope task my_task $end";
        let (remaining, scope_name) = parse_scope_declaration(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(scope_name, "my_task");
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

        // Capital X should also work
        let input = "XSIG1";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "SIG1");
        assert!(matches!(value, WaveValue::Binary(Value::VX)));

        let input = "zSIG2";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "SIG2");
        assert!(matches!(value, WaveValue::Binary(Value::VZ)));

        // Capital Z should also work
        let input = "ZSIG2";
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

        // Capital B should also work
        let input = "B00000000 %";
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

        // Test with capital X and Z values in bus
        let input = "b10XZ101Z %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "10XZ101Z"));
    }

    #[test]
    fn test_parse_value_change_real() {
        let input = "r1.234 %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "r1.234"));

        // Capital R should also work
        let input = "R1.234 %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "r1.234"));

        // Scientific notation
        let input = "r1.234e-5 %";
        let (remaining, (value, id)) = parse_value_change(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(id, "%");
        assert!(matches!(value, WaveValue::Bus(ref s) if s == "r1.234e-5"));
    }

    #[test]
    fn test_parse_simple_vcd() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write a simple VCD file
        writeln!(temp_file, "$date November 11, 2023 $end").unwrap();
        writeln!(temp_file, "$version Test VCD 1.0 $end").unwrap();
        writeln!(temp_file, "$timescale 1ps $end").unwrap();
        writeln!(temp_file, "$scope module test $end").unwrap();
        writeln!(temp_file, "$var wire 1 ! clk $end").unwrap();
        writeln!(temp_file, "$var wire 1 $ reset $end").unwrap();
        writeln!(temp_file, "$var wire 8 % data $end").unwrap();
        writeln!(temp_file, "$upscope $end").unwrap();
        writeln!(temp_file, "$enddefinitions $end").unwrap();
        writeln!(temp_file, "$dumpvars").unwrap();
        writeln!(temp_file, "0!").unwrap();
        writeln!(temp_file, "1$").unwrap();
        writeln!(temp_file, "b00000000 %").unwrap();
        writeln!(temp_file, "$end").unwrap();
        writeln!(temp_file, "#5").unwrap();
        writeln!(temp_file, "b00001111 %").unwrap();
        writeln!(temp_file, "#10").unwrap();
        writeln!(temp_file, "1!").unwrap();
        writeln!(temp_file, "b11110000 %").unwrap();
        writeln!(temp_file, "#15").unwrap();
        writeln!(temp_file, "b01010101 %").unwrap();
        writeln!(temp_file, "#20").unwrap();
        writeln!(temp_file, "0!").unwrap();
        writeln!(temp_file, "0$").unwrap();
        writeln!(temp_file, "b10101010 %").unwrap();

        // Parse the VCD file
        let vcd_data = parse_vcd_file(temp_file.path()).unwrap();

        // Check the parsed data
        assert_eq!(vcd_data.signals.len(), 3);

        // Check that signals are in the same order as they appear in the VCD file
        assert_eq!(vcd_data.signals[0], "test.clk");
        assert_eq!(vcd_data.signals[1], "test.reset");
        assert_eq!(vcd_data.signals[2], "test.data");

        assert_eq!(vcd_data.max_time, 20);

        // Check clk values
        let clk_values = vcd_data.values.get("test.clk").unwrap();
        assert_eq!(clk_values.len(), 3);
        assert_eq!(clk_values[0].0, 0);
        assert!(matches!(clk_values[0].1, WaveValue::Binary(Value::V0)));
        assert_eq!(clk_values[1].0, 10);
        assert!(matches!(clk_values[1].1, WaveValue::Binary(Value::V1)));
        assert_eq!(clk_values[2].0, 20);
        assert!(matches!(clk_values[2].1, WaveValue::Binary(Value::V0)));

        // Check data values
        let data_values = vcd_data.values.get("test.data").unwrap();
        assert_eq!(data_values.len(), 5);
        assert_eq!(data_values[0].0, 0);
        assert!(matches!(data_values[0].1, WaveValue::Bus(ref s) if s == "00"));
        assert_eq!(data_values[1].0, 5);
        assert!(matches!(data_values[1].1, WaveValue::Bus(ref s) if s == "0F"));
        assert_eq!(data_values[2].0, 10);
        assert!(matches!(data_values[2].1, WaveValue::Bus(ref s) if s == "F0"));
        assert_eq!(data_values[3].0, 15);
        assert!(matches!(data_values[3].1, WaveValue::Bus(ref s) if s == "55"));
        assert_eq!(data_values[4].0, 20);
        assert!(matches!(data_values[4].1, WaveValue::Bus(ref s) if s == "AA"));
    }
}
