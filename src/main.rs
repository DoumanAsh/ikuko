#![no_main]

use arg::Args;
use tokio::net::TcpListener;
use http_fs::config::{self, StaticFileConfig, TokioWorker};
use http_fs::StaticFiles;
use hyper::server::conn::http1;

use std::net;
use std::path::{Path, PathBuf};
use std::borrow::Cow;

mod io;

#[derive(Clone)]
pub struct DirectoryConfig {
    dir: Option<PathBuf>,
    dev_cors: bool,
    index_file: Option<Cow<'static, str>>
}

impl DirectoryConfig {
    fn new(dir: Option<PathBuf>, dev_cors: bool, index_file: Option<Cow<'static, str>>) -> Self {
        Self {
            dir,
            dev_cors,
            index_file,
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

    fn index_file(&self, _path: &Path) -> Option<&Path> {
        if let Some(index_file) = self.index_file.as_ref() {
            Some(Path::new(index_file.as_ref()))
        } else {
            None
        }
    }

    fn handle_directory(&self, _path: &Path) -> bool {
        true
    }

    fn on_response(&self, parts: &mut http_fs::http::response::Parts) {
        use http_fs::http;

        const TRUE: http::HeaderValue = http::HeaderValue::from_static("true");
        const WILDCARD: http::HeaderValue = http::HeaderValue::from_static("*");

        if self.dev_cors {
            use http::header::ACCESS_CONTROL_ALLOW_ORIGIN;
            use http::header::ACCESS_CONTROL_ALLOW_HEADERS;
            use http::header::ACCESS_CONTROL_ALLOW_METHODS;
            use http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS;

            parts.headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, WILDCARD);
            parts.headers.insert(ACCESS_CONTROL_ALLOW_METHODS, WILDCARD);
            parts.headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, WILDCARD);
            parts.headers.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, TRUE);
        }
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
                    0 => u16::MAX,
                    port => port,
                };
                continue
            }
        };

        break (addr, listener);
    }
}

#[derive(Debug, Args)]
#[arg(infer_name)]
///Static file server
pub struct Cli {
    #[arg(long, short, default_value = "8080")]
    ///Specifies port to use. If not available, tries another one until success. Default is 8080
    pub port: u16,
    ///Optionally specifies directory to server. By default is current directory.
    pub path: Option<PathBuf>,
    ///Specifies to allow CORS from any origin
    #[arg(long, default_value = "false")]
    pub dev_cors: bool,
    #[arg(long, default_value = "false")]
    ///Enables use of `index.html` instead of directory listing when hitting directory
    pub auto_index: bool,
}

async fn serve(listener: TcpListener, Cli { path, dev_cors, auto_index, .. }: Cli) {
    let index_file = if auto_index {
        Some("index.html".into())
    } else {
        None
    };

    let static_files = StaticFiles::new(TokioWorker, DirectoryConfig::new(path, dev_cors, index_file));
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

    tokio.block_on(serve(listener, args));
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
