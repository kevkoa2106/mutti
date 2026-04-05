use clap::{arg, command, value_parser};
use std::path::PathBuf;

pub struct Args {
    pub audio_file: String,
}

impl Args {
    pub fn parse() -> Args {
        let matches = command!()
            .arg(
                arg!([file] "File or directory to play")
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            )
            .get_matches();

        let file = matches
            .get_one::<PathBuf>("file")
            .expect("file argument is required");

        Args {
            audio_file: file.to_string_lossy().to_string(),
        }
    }
}
