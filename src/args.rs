use clap::{arg, command, value_parser};

pub struct Args {
    pub audio_file: &'static str,
}

pub fn get_args() -> Args {
    let matches = command!()
        .arg(
            arg!([file] "Optional specific file to play")
                .required(false)
                .value_parser(value_parser!(std::path::PathBuf)),
        )
        .get_matches();

    let file = matches.get_one::<&'static str>("file").unwrap();

    Args { audio_file: file }
}
