use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::path::Path;
use std::env;

use regex::Regex;

use unicode_segmentation::UnicodeSegmentation;

fn convert_gitignore_line(line: &str) -> Vec<String> {

    // Remove any leading/trailing whitespace
    let input_line = line.trim();

    // Skip empty lines and comments
    if input_line.is_empty() {
        return Vec::new();
    }

    // don't change comment lines
    if input_line.starts_with("#")
    {
        return vec![input_line.to_string()];
    }

    let mut output_lines : Vec<String> = vec![];

     // Expand character classes ("or" patterns) into separate lines
    //
    //     /[Ll]ibrary/
    // ----------------------------------
    //     /Library/
    //     /library/
    let char_class_regex = Regex::new(r"(\[(.*?)\])+?").unwrap();

    let matches = char_class_regex.find_iter(input_line);

    for m in matches
    {
        println!("Match {}", m.as_str());

        let char_class_str = m.as_str();

        let graphemes = UnicodeSegmentation::graphemes(char_class_str, true).collect::<Vec<&str>>();

        for grapheme in graphemes[1..graphemes.len()-1].iter()
        {
            println!("grapheme {}", grapheme);

            let mut line_with_unrolled_char_class=  input_line.to_string();
            line_with_unrolled_char_class.replace_range(m.range(), grapheme);


            println!("out_line {}", line_with_unrolled_char_class.as_str());

            output_lines.push(line_with_unrolled_char_class);
        }
    }

    // if there is a / before the end of the string, it's relative to the current directory
    let is_relative_to_ignore_file =  input_line.find('/') < Some(input_line.len() - 1);

    if is_relative_to_ignore_file {
        let mut out_line: String; 

        if input_line.starts_with('/')
        {
            out_line = input_line.to_string();
        }
        else
        {
            out_line = "/".to_owned();
            out_line.push_str(input_line);
        }

        output_lines.push(out_line);
    }


    return output_lines;

}

fn convert_gitignore_to_p4ignore(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {

    println!("Converting {} -> {}", 
             input_path.display(), 
             output_path.display());

    // Open input .gitignore file
    let input_file = File::open(input_path)?;
    let input_reader = BufReader::new(input_file);

    // Create output .p4ignore file
    let output_file = File::create(output_path)?;
    let mut output_writer = BufWriter::new(output_file);

    // Iterate through each line in the input file
    for line in input_reader.lines() {
        let line = line?;
        
        // Convert the line and write converted patterns
        for converted_line in convert_gitignore_line(&line) {
            writeln!(output_writer, "{}", converted_line)?;
        }
    }

    println!("Conversion complete: {} -> {}", 
             input_path.display(), 
             output_path.display());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Validate input arguments
    if args.len() != 3 {
        eprintln!("Usage: {} <input_gitignore_path> <output_p4ignore_path>", args[0]);
        std::process::exit(1);
    }

    // Convert paths
    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    // Perform conversion
    convert_gitignore_to_p4ignore(input_path, output_path)?;

    Ok(())
}