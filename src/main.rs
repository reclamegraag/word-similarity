use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::process;

use clap::{App, Arg};
use rayon::prelude::*;
use strsim::normalized_levenshtein;

use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};

/// The entry point of the Rust word similarity application.
fn main() {
    // Parse command-line arguments
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

    // Get input and output file paths
    let input_path = Path::new(matches.value_of("INPUT").unwrap());
    let output_path = Path::new(matches.value_of("OUTPUT").unwrap());

    // Parse minimum match percentage
    let min_match = matches
        .value_of("MIN_MATCH")
        .unwrap()
        .parse::<f64>()
        .unwrap_or(80.0) / 100.0;

    let start = Instant::now(); // Start the timer

    // Read input file and obtain a vector of word pairs
    let words = read_input_file(&input_path).expect("Error: Failed to read the input file.");

    // Calculate similarity matrix
    let similarity_matrix = calculate_similarity_matrix(&words);

    // Write similarity matrix to output file
    if let Err(e) = write_output_file(&output_path, &similarity_matrix, &words, min_match) {
        eprintln!("Error writing output file: {}", e);
        process::exit(1);
    }

    println!("Time elapsed: {:?}", start.elapsed()); // Print out the elapsed time
}

/// Read the input file and return a vector of word pairs.
fn read_input_file(
    input_path: &Path,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut words = Vec::new();
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    for (index, line) in reader.lines().enumerate() {
        let line = line?;

        // Check for empty lines
        if line.is_empty() {
            return Err("Empty lines are not allowed in the input file".into());
        }

        // Preprocess line and store original and processed word in a tuple
        let processed_line = line.to_lowercase();
        words.push((processed_line.clone(), line));

        progress_bar.set_length((index + 1) as u64);
    }

    progress_bar.finish();

    // Check for too few or too many words
    let word_count = words.len();
    const MIN_WORDS: usize = 2;
    const MAX_WORDS: usize = 500_000;
    if word_count < MIN_WORDS || word_count > MAX_WORDS {
        return Err(format!(
            "Invalid number of words: {}. The input file must contain between {} and {} words.",
            word_count, MIN_WORDS, MAX_WORDS
        )
        .into());
    }

    Ok(words)
}

/// Calculate the similarity matrix using the normalized Levenshtein distance.
fn calculate_similarity_matrix(words: &[(String, String)]) -> Vec<Vec<f64>> {
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    let res = words
        .par_iter()
        .map(|(processed_word1, _)| {
            progress_bar.set_length(words.len() as u64);
            progress_bar.inc(1);

            words
                .iter()
                .map(|(processed_word2, _)| {
                    normalized_levenshtein(&processed_word1, &processed_word2)
                })
                .collect::<Vec<f64>>()
        })
        .collect::<Vec<Vec<f64>>>();

    progress_bar.finish();
    res
}

/// Write the similarity matrix to the output file.
fn write_output_file(
    path: &Path,
    matrix: &[Vec<f64>],
    words: &[(String, String)],
    min_match: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Collect all pairs in a vector
    let mut all_pairs: Vec<(f64, usize, &str, usize, &str)> = Vec::new();

    for (i, row) in matrix.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            if i >= j {
                continue; // Skip if the index of the first word is greater than or equal to the index of the second word
            }
            if *value >= min_match {
                let (_, original_word1) = &words[i];
                let (_, original_word2) = &words[j];
                all_pairs.push((*value, i + 1, original_word1, j + 1, original_word2));
            }
        }
    }

    // Sort pairs in descending order of similarity
    all_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Write sorted pairs to the output file
    for pair in all_pairs {
        writeln!(
            writer,
            "Row {}: {}\tRow {}: {}\tSimilarity: {:.2}%",
            pair.1,
            pair.2,
            pair.3,
            pair.4,
            pair.0 * 100.0
        )?;
    }

    Ok(())
}
