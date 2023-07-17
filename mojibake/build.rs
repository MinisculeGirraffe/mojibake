use std::fs::File;
use std::io::{BufRead, BufWriter, Write};
use std::path::Path;
use std::{env, io};

fn main() {
    let cwd = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());
    let mut emoji_map = phf_codegen::OrderedMap::<u16>::new();
    let mut number_map = phf_codegen::OrderedMap::new();
    let mut tail_map = phf_codegen::OrderedMap::<u16>::new();
    let mut tail_number_map = phf_codegen::OrderedMap::new();
    let emoji_path = Path::new(&cwd).join("emoji-sequences.txt");
    let emoji_file = File::open(emoji_path).unwrap();
    //These alone will goof up the grapheme boundary and cause decoding issues.
    let forbidden_chars = vec!["🏻", "🏼", "🏽", "🏾", "🏿"];
    let lines: Vec<_> = io::BufReader::new(emoji_file)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let mut emoji_list: Vec<String> = Vec::new();

    for line in lines.into_iter().rev() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        let code_points_field = line.split(';').next().unwrap().trim();

        // Check if this is a range of code points.
        if code_points_field.contains("..") {
            // Parse the start and end of the range.
            let range_parts: Vec<&str> = code_points_field.split("..").collect();
            let start = u32::from_str_radix(range_parts[0], 16).unwrap();
            let end = u32::from_str_radix(range_parts[1], 16).unwrap();
            // Add each code point in the range to the map.
            for code in start..=end {
                if let Some(ch) = char::from_u32(code) {
                    emoji_list.push(ch.to_string());
                }
            }
        } else {
            // This is a single code point or sequence of code points.
            let emoji_string: String = code_points_field
                .split_whitespace()
                .filter_map(|code| u32::from_str_radix(code, 16).ok())
                .filter_map(char::from_u32)
                .collect();

            emoji_list.push(emoji_string);
        }
    }

    // Now you can index into emoji_map to get an emoji by its number, and find the number of an emoji with .iter().position().
    for (index, emoji) in emoji_list
        .iter()
        .filter(|i| !forbidden_chars.contains(&i.as_str()))
        .enumerate()
    {
        if index < 2048 {
            emoji_map.entry(index as u16, format!(r#""{emoji}""#).as_str());
            number_map.entry(emoji.as_str(), format!("{index}").as_str());
        } else {
            let offset_index = (index - 2048) as u16;
            tail_map.entry(offset_index, format!(r#""{emoji}""#).as_str());
            tail_number_map.entry(emoji.as_str(), format!("{offset_index}").as_str());
        }
    }

    write!(
        &mut file,
        "pub static EMOJI_MAP: phf::ordered_map::OrderedMap<u16,&'static str > = {}",
        emoji_map.build()
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap();
    write!(
        &mut file,
        "pub static NUMBER_MAP: phf::ordered_map::OrderedMap<&'static str,u16> = {}",
        number_map.build()
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap();

    write!(
        &mut file,
        "pub static TAIL_MAP: phf::ordered_map::OrderedMap<u16,&'static str > = {}",
        tail_map.build()
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap();
    write!(
        &mut file,
        "pub static TAIL_NUMBER_MAP: phf::ordered_map::OrderedMap<&'static str,u16> = {}",
        tail_number_map.build()
    )
    .unwrap();
    writeln!(&mut file, ";\n").unwrap();
}
