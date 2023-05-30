use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path};
use std::process;

use clap::{App, Arg};
use rayon::prelude::*;
use strsim::normalized_levenshtein;

use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

/// The entry point of the Rust word similarity application.
///
/// The application reads an input file containing a list of words, calculates the
/// similarity matrix by computing the normalized Levenshtein distance between each
/// pair of words, and writes the resulting similarity matrix to an output file.
///
/// The application uses the `clap` crate for command-line argument parsing and the
/// `rayon` crate to parallelize the computation of the similarity matrix.
///
/// # Panics
///
/// The application will panic if the command-line arguments are not provided or are invalid.
fn main() {
    let matches = App::new("Word Similarity")
        .version("0.1.0")
        .author("Roderik von Maltzahn")
        .about("Calculates similarity percentages between word pairs")
        .arg(
            Arg::with_name("INPUT")
                .help("Input file containing a list of words")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Output file for similarity percentages")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("MIN_MATCH")
                .help("Minimum match percentage (default: 80)")
                .short('m')
                .long("min-match")
                .takes_value(true)
                .default_value("80"),
        )
        .get_matches();

    let input_path = Path::new(matches.value_of("INPUT").unwrap());
    let output_path = Path::new(matches.value_of("OUTPUT").unwrap());
    let min_match = matches
        .value_of("MIN_MATCH")
        .unwrap()
        .parse::<f64>()
        .unwrap_or(80.0) / 100.0;

    let start = Instant::now(); // Start the timer

    let words = read_input_file(&input_path).expect("Error: Failed to read the input file.");

    let similarity_matrix = calculate_similarity_matrix(&words);

    if let Err(e) = write_output_file(&output_path, &similarity_matrix, &words, min_match) {
        eprintln!("Error writing output file: {}", e);
        process::exit(1);
    }

    println!("Time elapsed: {:?}", start.elapsed()); // Print out the elapsed time
}

/// Read the input file and return a vector of words.
///
/// The function reads an input file line by line, where each line represents a word.
/// It returns a vector containing the words. If there is an error while reading the file,
/// the function will return an appropriate error message.
///
/// # Arguments
///
/// * `input_file_path` - A reference to the input file path as a `PathBuf`.
///
/// # Returns
///
/// A `Result` containing a vector of `String` representing the words or an error.
///
/// # Errors
///
/// This function will return an error if the input file cannot be read.
const MIN_WORDS: usize = 2;
const MAX_WORDS: usize = 500_000;

fn read_input_file(input_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut words = Vec::new();
    let progress_bar = ProgressBar::new(MAX_WORDS as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    for line in reader.lines() {
        let line = line?;

        // Check for empty lines
        if line.is_empty() {
            return Err("Empty lines are not allowed in the input file".into());
        }

        let processed_line = line.to_lowercase().replace(" ", ""); // Preprocessing line
        words.push(processed_line);

        progress_bar.inc(1);
    }

    progress_bar.finish();

    // Check for too few or too many words
    let word_count = words.len();
    if word_count < MIN_WORDS || word_count > MAX_WORDS {
        return Err(format!(
            "Invalid number of words: {}. The input file must contain between {} and {} words.",
            word_count, MIN_WORDS, MAX_WORDS
        ).into());
    }

    Ok(words)
}

fn calculate_similarity_matrix(words: &[String]) -> Vec<Vec<f64>> {
    let progress_bar = ProgressBar::new(words.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    let res = words
        .par_iter()
        .map(|word1| {
            progress_bar.inc(1);
            words
                .iter()
                .map(|word2| normalized_levenshtein(word1, word2))
                .collect::<Vec<f64>>()
        })
        .collect::<Vec<Vec<f64>>>();

    progress_bar.finish();
    res
}

/// Write the similarity matrix to the output file in the specified format.
///
/// The function writes the similarity matrix to the output file, where each row represents
/// the similarity values between a word and all other words in the input list. If there is an
/// error while writing to the output file, the function will return an appropriate error message.
///
/// # Arguments
///
/// * `output_file_path` - A reference to the output file path as a `PathBuf`.
/// * `similarity_matrix` - A reference to the similarity matrix as a `Vec<Vec<f64>>`.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
///
/// # Errors
///
/// This function will return an error if the output file cannot be written.
fn write_output_file(
    path: &Path,
    matrix: &[Vec<f64>],
    words: &[String],
    min_match: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Collect all pairs in a vector
    let mut all_pairs: Vec<(f64, usize, String, usize, String)> = Vec::new();

    for (i, row) in matrix.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            if i >= j {
                continue; // Skip if index of first word is greater than or equal to index of second word
            }
            if *value >= min_match {
                all_pairs.push((*value, i + 1, words[i].clone(), j + 1, words[j].clone()));
            }
        }
    }

    // Sort pairs in descending order of similarity
    all_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Write sorted pairs to the output file
    for pair in all_pairs {
        writeln!(
            writer,
            "Row {}: {} ~ Row {}: {} | Similarity: {:.2}%",
            pair.1,
            pair.2,
            pair.3,
            pair.4,
            pair.0 * 100.0
        )?;
    }

    Ok(())
}
