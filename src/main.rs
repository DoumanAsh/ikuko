use http_fs::config::{self, StaticFileConfig};
use http_fs::{StaticFiles};
use futures::Future;

use std::path::{Path, PathBuf};

mod cli;

//Note that Clone is required to share StaticFiles among hyper threads
#[derive(Clone)]
pub struct DirectoryConfig {
    dir: Option<PathBuf>,
}

impl DirectoryConfig {
    fn new(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
        }
    }
}

impl StaticFileConfig for DirectoryConfig {
    type FileService = config::DefaultConfig;
    type DirService = config::DefaultConfig;

    fn serve_dir(&self) -> &Path {
        match self.dir.as_ref() {
            Some(dir) => dir,
            None => config::DefaultConfig.serve_dir(),
        }
    }

    fn handle_directory(&self, _path: &Path) -> bool {
        true
    }
}

fn run() -> Result<(), i32> {
    let args = cli::Args::new(std::env::args().skip(1))?;

    let mut port = args.port;

    let (addr, server) = loop {
        let addr = ([127, 0, 0, 1], port).into();

        let server = match hyper::Server::try_bind(&addr) {
            Ok(server) => server,
            Err(_) => {
                port = match port.wrapping_sub(1) {
                    0 => core::u16::MAX,
                    port => port,
                };
                continue
            }
        };

        break (addr, server);
    };

    let static_files = StaticFiles::new(DirectoryConfig::new(args.path));

    let server = server.serve(static_files).map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(exit_code) => std::process::exit(exit_code)
    }
}
