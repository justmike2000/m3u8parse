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

Supports fetching media tags, media resources, and variant streams.  Provide a key to sort the results by:

```
let media_tags = parsed.get_media_tags("CHANNELS");

let media_resources = parsed.get_media_resources("BANDWIDTH");

let variant_streams = parsed.get_variant_streams("BANDWIDTH");
```

```
println!("{:?}", variant_streams);

[{"BANDWIDTH": "10429877", "FRAME-RATE": "23.97", "RESOLUTION": "1920x1080", "CLOSED-CAPTIONS": "NONE", "CODECS": "ec-3", "AVERAGE-BANDWIDTH": "6996616", "AUDIO": "atmos", "uri": "hdr10/unenc/6000k/vod.m3u8", "VIDEO-RANGE": "PQ"}, {"AUDIO": "aac-128k", "CLOSED-CAPTIONS": "NONE", "BANDWIDTH": "12156778", "AVERAGE-BANDWIDTH": "7766087", "RESOLUTION": "1920x1080", "VIDEO-RANGE": "PQ", "FRAME-RATE": "23.97", "uri": "hdr10/unenc/7700k/vod.m3u8", "CODECS": "mp4a.40.2"}]
```


## Build

```
cargo build
```

## Tests

```
cargo test
```
