use std::path::{Path, PathBuf};

const USAGE: &str = "Static file server
USAGE:
    ikuko [options] [path]

OPTIONS:
    -p, --port - Specifies port to use. If not available, tries another one until success. Default is 65535.
    -h, --help - Prints this help message.

ARGS:
    [path] - Optionally specifies directory to server. By default is current directory.
";

#[derive(Debug)]
pub struct Args {
    pub path: Option<PathBuf>,
    pub port: u16,
}

impl Args {
    pub fn new<A: Iterator<Item=String>>(mut args: A) -> Result<Self, i32> {
        let mut path = None;
        let mut port = core::u16::MAX;

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                match &arg[1..]{
                    "h" | "-help" => {
                        println!("{}", USAGE);
                        return Err(0);
                    },
                    "p" | "-port" => match args.next() {
                        Some(arg) => match str::parse(&arg) {
                            Ok(new_port) if new_port > 0 => {
                                port = new_port;
                            },
                            _ => {
                                eprintln!("Invalid port is specified. Should be positive integer within range [1..65535]");
                                return Err(3);
                            },
                        },
                        None => {
                            eprintln!("Flag {} is specified, but suffix is missing", arg);
                            return Err(2);
                        },
                    }
                    _ => {
                        eprintln!("Invalid flag '{}' is specified", arg);
                        println!("{}", USAGE);
                        return Err(2);
                    }
                }
            } else if path.is_none() {
                path = Some(Path::new(&arg).to_path_buf());
            } else {
                eprintln!("Multiple paths, which one you want to serve? You can tell me only one.");
                return Err(3);
            }
        }

        Ok(Self {
            path,
            port,
        })
    }
}
