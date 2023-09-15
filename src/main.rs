#![no_main]

use arg::Args;
use tokio::net::TcpListener;
use http_fs::config::{self, StaticFileConfig, TokioWorker};
use http_fs::StaticFiles;
use hyper::server::conn::http1;

use std::net;
use std::path::{Path, PathBuf};

mod io;

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

async fn listen(mut port : u16) -> (net::SocketAddr, TcpListener) {
    loop {
        let addr = net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), port);

        let listener = match TcpListener::bind(addr).await {
            Ok(server) => server,
            Err(error) => {
                eprintln!("{addr}: Cannot bind: {error}");
                port = match port.wrapping_sub(1) {
                    0 => core::u16::MAX,
                    port => port,
                };
                continue
            }
        };

        break (addr, listener);
    }
}

#[derive(Debug, Args)]
///Static file server
pub struct Cli {
    #[arg(long, short, default_value = "8080")]
    ///Specifies port to use. If not available, tries another one until success. Default is 8080
    pub port: u16,
    ///Optionally specifies directory to server. By default is current directory.
    pub path: Option<PathBuf>,
}

async fn serve(listener: TcpListener, path: Option<PathBuf>) {
    let static_files = StaticFiles::new(TokioWorker, DirectoryConfig::new(path));
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(result) => result,
            Err(error) => {
                eprintln!(">Failed to accept incoming TCP connection: {error}");
                continue;
            }
        };
        let stream = io::IoWrapper(stream);
        let static_files = static_files.clone();
        tokio::task::spawn(async move {
            let result = http1::Builder::new().keep_alive(false).serve_connection(stream, static_files).await;
            if let Err(error) = result {
                eprintln!(">{error}");
            }
        });
    }
}

fn run(args: Cli) -> Result<(), isize> {
    let tokio = match tokio::runtime::Builder::new_current_thread().enable_io().build() {
        Ok(tokio) => tokio,
        Err(error) => {
            eprintln!("Cannot create IO runtime: {error}");
            return Err(2);
        }
    };
    let (addr, listener) = tokio.block_on(listen(args.port));
    println!("Listening on http://{}", addr);

    tokio.block_on(serve(listener, args.path));
    Ok(())
}

#[no_mangle]
pub fn rust_main(args: c_main::Args) -> isize {
    match Cli::from_args(args.into_iter().skip(1)) {
        Ok(args) => match run(args) {
            Ok(()) => 0,
            Err(code) => code,
        },
        Err(arg::ParseKind::Sub(name, arg::ParseError::HelpRequested(help))) => {
            println!("{name}: {}", help);
            0
        },
        Err(arg::ParseKind::Top(arg::ParseError::HelpRequested(help))) => {
            println!("{}", help);
            0
        },
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    }
}
