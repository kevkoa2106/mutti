use clap::{arg, command, value_parser};
use std::path::PathBuf;

pub struct Args {
    pub audio_file: Option<String>,
    pub visualize: bool,
}

impl Args {
    pub fn parse() -> Args {
        let matches = command!()
            .arg(
                arg!([file] "File or directory to play")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(arg!(--visualize "Enable the audio visualizer"))
            .get_matches();

        let file = matches
            .get_one::<PathBuf>("file")
            .map(|f| f.to_string_lossy().to_string());

        Args {
            audio_file: file,
            visualize: matches.get_flag("visualize"),
        }
    }
}
