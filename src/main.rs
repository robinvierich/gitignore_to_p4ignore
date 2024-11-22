use std::collections::btree_map::Range;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::path::Path;
use std::{env, vec};

use itertools::{enumerate, Itertools};

use regex::{Match, Regex};

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

    let matches = char_class_regex.find_iter(input_line).collect::<Vec<Match>>();
    let match_ranges = matches.iter().map(|m| m.range()).collect::<Vec<core::ops::Range<usize>>>();

    let mut match_chars: Vec<Vec<&str>> = vec![];

    for m in matches.iter()
    {
        println!("Match {}", m.as_str());

        let char_class_str = m.as_str();
        let graphemes = UnicodeSegmentation::graphemes(char_class_str, true).collect::<Vec<&str>>();

        let stripped_graphemes = graphemes[1..graphemes.len()-1].to_vec();

        println!("stripped_graphemes {:?}", stripped_graphemes);

        //match_chars.push((m.range(), graphemes[1..graphemes.len()-1].iter()));
        match_chars.push(stripped_graphemes);
    }


    //let num_options = match_chars.iter().fold(1, |acc, cs| acc * cs.len());
    //let options = match_chars.iter().map(|x| (x.0, x.1)).multi_cartesian_product();
    //let options = match_chars.iter().map(|x| x.into_iter()).multi_cartesian_product();

    //let mut queue = vec![];

    //if (!match_chars.is_empty())
    //{
    //    queue = match_chars.first().unwrap().to_vec();
    //}

    // [[l, L], [n, N, m, M], [p, P]]
    // -->
    // [l, n, p]
    // [l, n, P]
    // [l, N, p]
    // [l, N, P]
    // [l, m, p]
    // [l, m, P]
    // [l, M, p]
    // [l, M, P]
    // [L, n, p]
    // [L, n, P]
    // [L, N, p]
    // [L, N, P]
    // [L, m, p]
    // [L, m, P]
    // [L, M, p]
    // [L, M, P]

    // for a = l, L
    //
    //  for ia = 0..[l, L].num()
    //  for ia = 0..dims[dim].num()
    //
    // for b = n, N, m, M
    // for c = p, P


    //let mut dim_idxs : Vec<usize> = vec![0; dim_strides.len()];

    // fn is_iteration_complete(idxs: &Vec<usize>, strides: &Vec<usize>) -> bool
    // {
    //     if idxs.is_empty()
    //     {
    //         return true;
    //     }

    //     return idxs[0] >= strides[0];
    // }

    // fn step_iteration(idxs: &mut Vec<usize>, strides: &Vec<usize>)
    // {
    //     // iterate
    //     for i_dim in (0..idxs.len()).rev()
    //     {
    //         // if this is the last dim, always wrap.
    //         // if this is not the last dimension, and the next dimension wrapped back to zero, carry that increment to this dimension
    //         let is_last_dimension = i_dim == idxs.len() - 1;
    //         if is_last_dimension || idxs[i_dim + 1] == 0
    //         {
    //             // #hack let 1st dimension overflow so we can detect completion..
    //             let is_first_dimension = i_dim == 0;
    //             if is_first_dimension
    //             {
    //                 idxs[i_dim] += 1;
    //             }
    //             else
    //             {
    //                 idxs[i_dim] = (idxs[i_dim] + 1) % strides[i_dim];
    //             }
    //         }
    //     }
    // }


    let strides = match_chars.iter().map(|x| x.len()).collect::<Vec<usize>>();

    let mut wrap_values = strides.clone();

    if !strides.is_empty()
    {
        for i in (0..(strides.len()-1)).rev()
        {
            wrap_values[i] = strides[i] * strides[i + 1];
        }
    }

    fn get_indices_for_iteration(iteration_index: usize, wrap_values: &Vec<usize>) -> Vec<usize>
    {
        let mut indices = vec![0; wrap_values.len()];
        
        for i_dim in (0..indices.len()).rev()
        {
            // if this is the last dim, always wrap.
            // if this is not the last dimension, and the next dimension wrapped back to zero, carry that increment to this dimension
            let is_last_dimension = i_dim == indices.len() - 1;
            if is_last_dimension 
            {
                indices[i_dim] = iteration_index % wrap_values[i_dim];
            }
            else
            {
                indices[i_dim] = iteration_index / (wrap_values[i_dim + 1]);
            }
        }

        return indices;

        // for i in 0..wrap_values.len() {
            
        // }
        // return wrap_values.iter().map(|wrap_val| iteration_index % wrap_val).collect();
    }

    let num_iterations = if wrap_values.is_empty() { 0 } else { wrap_values[0] };

    println!("converting {:?}, num iters: {}, wrap_values: {:?}", match_chars, num_iterations, wrap_values);

    for i in 0..num_iterations {
        let indices =  get_indices_for_iteration(i, &wrap_values);

        println!("indices: {:?}", indices);

        let char_combo = enumerate(&indices).map(|(i_dim, i_char)| match_chars[i_dim][*i_char]).collect::<Vec<&str>>();

        println!("replacing chars {:?}", char_combo.iter().collect::<Vec<&&str>>()); //.map(|c| c.to_string()).collect::<String>());

        let mut unrolled_line=  input_line.to_string();

        let mut range_offset : usize = 0;

        for (i, c) in enumerate(char_combo)
        {
            let r = std::ops::Range {
                start: &match_ranges[i].start - range_offset, 
                end: &match_ranges[i].end - range_offset
            };

            range_offset += if r.len() > 0 { r.len() - 1 } else { 0 };

            unrolled_line.replace_range(r, c);

        }

        println!("\tin:  {}\n\tout: {}", input_line, unrolled_line.as_str());
    }


    // while !is_iteration_complete(&dim_idxs, &dim_strides)
    // {
    //     //let curr_char_idxs = &dim_idxs; 

    //     let option = enumerate(&dim_idxs).map(|(i_dim, i_char)| match_chars[i_dim][*i_char]).collect::<Vec<&str>>();
    //     println!("option {:?}", option.iter().collect::<Vec<&&str>>()); //.map(|c| c.to_string()).collect::<String>());

    //     let mut unrolled_line=  input_line.to_string();

    //     for (i, c) in enumerate(option)
    //     {
    //         let r = &match_ranges[i];
    //         println!("grapheme {}", c);

    //         unrolled_line.replace_range(r.clone(), c);

    //         println!("out_line {}", unrolled_line.as_str());
    //     }

    //     output_lines.push(unrolled_line);

    //     step_iteration(&mut dim_idxs, &dim_strides);
    // }

  


    // num dims = matches.len()
    //
    // for idim in numdims
    //    for 
    //




    // for cs in options 
    // {
    //     let mut unrolled_line=  input_line.to_string();

    //     for (i, c) in enumerate(cs)
    //     {
    //         println!("grapheme {}", c);

    //         let r = match_ranges[i%match_ranges.len()].clone();

    //         unrolled_line.replace_range(r, c);

    //         println!("out_line {}", unrolled_line.as_str());
    //     }

    //     output_lines.push(unrolled_line);
    // }


    // if there is a / before the end of the string, it's relative to the current directory
    let is_relative_to_ignore_file =  input_line.find('/') < Some(input_line.len() - 1);

    if is_relative_to_ignore_file 
    {
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