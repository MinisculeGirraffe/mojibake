use mojibake::{decode, encode};
use unicode_segmentation::UnicodeSegmentation;
fn main() {
    let data = "Input, but as emoji";
    println!("Original Text:");
    print_info(data);
    let encoded = encode(data);
    println!("Mojibake Encoded:");
    print_info(&encoded);
    let decoded = decode(encoded).unwrap();
    println!("Decoded Text:");
    print_info(&String::from_utf8(decoded).unwrap())
}

fn print_info(data: &str) {
    println!(
        "\tValue: {data}\n\tBytes: {},\n\tCharacters: {},\n\tGraphemes: {}\n",
        data.len(),
        data.chars().count(),
        data.graphemes(true).count()
    )
}
