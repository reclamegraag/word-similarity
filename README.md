# Word Similarity

`Word Similarity` is a Rust application that calculates the similarity percentages between word pairs provided in an input file. The similarities are computed using the normalized Levenshtein distance.

## Requirements

- Rust Programming Language
- Cargo (Rust's build system and package manager)

## How to Use

1. **Compile the application:** 

   Open your terminal and navigate to the directory containing the application. Run the following command to compile the application:

   ```
   cargo build --release
   ```

   This command will create an executable in the `target/release` directory.

2. **Run the application:** 

   Run the application with the following command:

   ```
   ./target/release/word_similarity [INPUT] [OUTPUT] [OPTIONS]
   ```

   Replace `[INPUT]` with the path to your input file, and replace `[OUTPUT]` with the path where you want the output file to be written.

   The `OPTIONS` parameter is optional. The available option is:

   - `-m` or `--min-match` : Specify the minimum match percentage. The default value is 80.

   Here is an example of how to run the application:

   ```
   ./target/release/word_similarity words.txt similarities.txt --min-match 85
   ```

   This will calculate similarities for the words in the `words.txt` file and write the results to the `similarities.txt` file. Only pairs with a similarity percentage of 85% or higher will be included.

## Input File Format

The input file should contain one or multiple words per line. Empty lines are not allowed in the input file. Here is an example of the expected format:

```
hello world
apple orange banana
```

## Output File Format

The output file contains a similarity matrix with the calculated similarity percentages for each word pair. Here is an example of the output format:

```
Row 1: hello world ~ Row 2: apple orange banana | Similarity: 01.01%
```

## Contributing

We appreciate your help! Please feel free to submit pull requests with any improvements or bug fixes you make to this project.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
