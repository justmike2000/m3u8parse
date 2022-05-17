# m3u8parse

Library used to parse M3U8 playlist.   Currently only supports from_uri.

## Dependency 
In your Cargo.toml:
```
[dependencies]
m3u8parse = { git = "https://github.com/justmike2000/m3u8parse/" }
```

## Usage:

```
use m3u8parse::M3U8;

let uri = "http://<domain>/path/playlist.m3u8";

let parsed_m3u8 = M3U8::from_uri(uri).unwrap();
```

## Build

```
cargo build
```

## Tests

```
cargo test
```