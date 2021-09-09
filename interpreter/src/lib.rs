use std::{fs, error::Error};

use lexer::Lexer;
use parser::Parser;

pub use file_config::Config;

pub mod file_config {
    use std::{env, path::PathBuf, ffi::OsString};

    /// File configuration struct used for verifying environment arguments and storing a filename.
    pub struct Config {
        pub input_filename: PathBuf,
        pub output_filename: PathBuf,
    }

    impl Config {
        /// Creates a new file configuration struct using arguments from the command line
        /// as the file info. Command line must have executable name followed by the filename. An
        /// optional output filename can be added in addition to the input filename.
        /// 
        /// # Errors
        /// 
        /// This function will error if no filename is provided.
        pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
            args.next();

            match Config::extract_filenames(args.next(), args.next()) {
                Ok(names) => Ok(Config { input_filename: names.0, output_filename: names.1 }),
                Err(e) => Err(e)
            }
        }

        /// Retrieves the input and output filenames from the provided `Option` parameters. The output
        /// filename is optional.
        /// 
        /// # Errors
        /// 
        /// This function will error if no filename is provided.
        fn extract_filenames(input: Option<String>, output: Option<String>) -> Result<(PathBuf, PathBuf), &'static str> {
            // check if a filename was provided
            let input_path = match input {
                Some(filename) => PathBuf::from(filename),
                None => return Err("No filename was provided."),
            };

            // check if an output filename was provided
            let mut output_path = match output {
                Some(filename) => PathBuf::from(filename),
                // if no name was provided, create one from the input filename
                None => PathBuf::from(
                    match input_path.file_stem() {
                        Some(name) => {
                            let mut n = OsString::from(name);
                            n.push("-output");
                            n
                        },
                        None => OsString::from("output"),
                    }
                ),
            };

            // the output file extension will always be a .txt
            output_path.set_extension("txt");

            Ok((input_path, output_path))
        }
    }
}

/// Runs the file configuration and reads the provided filename's contents.
/// 
/// # Errors
/// 
/// This function will error if the file cannot be read, there is an issue generating tokens, or the tokens
/// cannot be parsed.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Reading contents from {:?}.", config.input_filename);

    let file_contents = fs::read_to_string(config.input_filename)?;

    println!("Generating tokens...");

    let mut lex = Lexer::new(file_contents);
    let tokens = lex.generate_tokens()?;

    println!("Generating tabs...");

    let mut par = Parser::new(tokens);
    let tabs = par.generate_tabs()?;

    // println!("{}", tabs);

    println!("Writing output to {:?}.", config.output_filename);

    fs::write(config.output_filename, tabs)?;

    println!("Guitar tabs interpreted successfully!");

    Ok(())
}