# ikuko

Simple static file server.

## Usage

```
Static file server

USAGE: [OPTIONS] [path]

OPTIONS:
    -h,  --help         Prints this help information
    -p,  --port <port>  Specifies port to use. If not available, tries another one until success. Default is 8080
         --dev_cors     Specifies to allow CORS from any origin
         --auto_index   Enables use of `index.html` instead of directory listing when hitting directory

ARGS:
    [path]  Optionally specifies directory to server. By default is current directory.
```
