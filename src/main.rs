use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::path::Path;
use std::{env, vec};

use itertools::enumerate;

use regex::{Match, Regex};

use unicode_segmentation::UnicodeSegmentation;

// Expand character classes ("or" patterns) into separate lines
//
//      /[Aa]ssets/[Ss]treamingAssets/aa.meta
// ----------------------------------
//      /Assets/StreamingAssets/aa.meta
//      /Assets/streamingAssets/aa.meta
//      /assets/StreamingAssets/aa.meta
//      /assets/streamingAssets/aa.meta
//
fn expand_character_classes_to_new_lines(input_line: &str) -> Vec<String> {
    let mut output_lines : Vec<String> = vec![];

    let char_class_regex = Regex::new(r"(\[(.*?)\])+?").unwrap();

    let matches = char_class_regex.find_iter(input_line).collect::<Vec<Match>>();
    if matches.is_empty() {
       return output_lines; 
    }


    let match_ranges = matches.iter().map(|m| m.range()).collect::<Vec<core::ops::Range<usize>>>();

    let mut match_graphemes: Vec<Vec<&str>> = vec![];

    for m in matches.iter()
    {
        println!("Match {}", m.as_str());

        let graphemes = UnicodeSegmentation::graphemes(m.as_str(), true).collect::<Vec<&str>>();

        let graphemes_excluding_brackets = graphemes[1..graphemes.len()-1].to_vec();

        match_graphemes.push(graphemes_excluding_brackets);
    }


    let strides = match_graphemes.iter().map(|x| x.len()).collect::<Vec<usize>>();

    let mut wrap_values = strides.clone();
    for i in (0..(strides.len()-1)).rev()
    {
        println!("i: {}, stride[{}]: {}, stride[{}]: {}", i,  i, strides[i], i+1, strides[i+1]);

        wrap_values[i] = strides[i] * wrap_values[i + 1];

        println!("i: {}, wrap_values[{}]: {}, wrap_values[{}]: {}", i, i, wrap_values[i], i+1, wrap_values[i+1]);
    }

    fn get_indices_for_iteration(iteration_index: usize, wrap_values: &Vec<usize>, strides: &Vec<usize>) -> Vec<usize>
    {
        let mut indices = vec![0; wrap_values.len()];
        
        for i_dim in (0..indices.len()).rev()
        {
            let next_dim_wrap_val = if (i_dim == indices.len() - 1) { 1 } else { wrap_values[i_dim + 1] };

            indices[i_dim] = (iteration_index / next_dim_wrap_val) % strides[i_dim];
        }

        return indices;
    }

    let num_iterations = wrap_values[0];

    println!("converting {:?}, num_iterations: {}, wrap_values: {:?}", match_graphemes, num_iterations, wrap_values);

    for i in 0..num_iterations {
        let indices =  get_indices_for_iteration(i, &wrap_values, &strides);

        let grapheme_combo = enumerate(&indices).map(|(i_dim, i_char)| match_graphemes[i_dim][*i_char]).collect::<Vec<&str>>();
        println!("using grapheme combo {:?}", grapheme_combo.iter()); 

        let mut unrolled_line=  input_line.to_string();

        let mut num_characters_removed : usize = 0;

        for (i, grapheme) in enumerate(grapheme_combo)
        {
            let r = std::ops::Range {
                start: &match_ranges[i].start - num_characters_removed, 
                end: &match_ranges[i].end - num_characters_removed
            };

            // r is guaranteed to be larger than grapheme because it includes the "[]" characters
            num_characters_removed += r.len() - grapheme.len(); 

            unrolled_line.replace_range(r, grapheme);
        }

        output_lines.push(unrolled_line);
    }

    return output_lines;

}


fn convert_gitignore_line(line: &str) -> Vec<String> {

    // Remove any leading/trailing whitespace
    let input_line = line.trim();

    // Skip empty lines 
    if input_line.is_empty() {
        return vec!["".to_owned()];
    }

    let mut output_lines = expand_character_classes_to_new_lines(input_line);

    if output_lines.is_empty() { output_lines = vec![input_line.to_string()] }

    println!("\tin:  {:?}\n\tout: {:?}", input_line, output_lines);

    for out_line in &mut output_lines {
        // exit early for comment lines
        if out_line.starts_with("#")
        {
            continue;
        }

        // if there is a / before the end of the string, it's relative to the current directory
        let is_relative_to_ignore_file = match out_line.find('/')
        {
            Some(pos) if pos < out_line.len() - 1 => true,
            _ => false
        };

        if is_relative_to_ignore_file 
        {
            if !out_line.starts_with('/')
            {
                let mut out_line_with_slash = "/".to_owned();
                out_line_with_slash.push_str(out_line);
                *out_line = out_line_with_slash;
            }
        }
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