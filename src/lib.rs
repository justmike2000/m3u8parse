//! A library to parse m3u8 playlist.
//!
//! # Examples
//!
//! <TODO>
//!
use std::collections::HashMap;

#[derive(Debug)]
pub enum ParseError {
    InvalidM3U8(String),
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for ParseError {
    fn from(err: reqwest::Error) -> ParseError {
        ParseError::ReqwestError(err)
    }
}

#[derive(Default, Debug)]
pub struct M3U8 {
    independent_segments: bool,
    version: String,
    media_tags: Vec<HashMap<String, String>>,
    variant_streams: Vec<HashMap<String, String>>,
    media_resources: Vec<HashMap<String, String>>,
}

impl M3U8 {
    fn new() -> Self {
        M3U8 {
            version: "2".to_string(),
            ..Default::default()
        }
    }

    fn validate(lines: &[String]) -> Result<(), ParseError> {
        let intro = lines
            .first()
            .ok_or_else(|| ParseError::InvalidM3U8("Invalid M3U8 format".to_string()))?;

        if intro != "#EXTM3U" {
            return Err(ParseError::InvalidM3U8("Missing #EXTM3U".to_string()));
        }

        if lines.iter().filter(|&n| *n == "EXT-X-VERSION").count() > 1 {
            return Err(ParseError::InvalidM3U8("Invalid M3U8, multiple version tags found.".to_string()));
        }
        
        Ok(())
    }

    fn get_key_value_pair(item: &str) -> Option<(String, String)> {
        let mut attr = item.split('=');
        let key = match attr.next() {
            Some(key) => key,
            None => return None
        };
        let value = match attr.next() {
            Some(value) => value,
            None => return None
        };
        return Some((key.to_string(), value.to_string()))
    }

    fn by_attribute(&mut self, data: &str) -> HashMap<String, String> {
        let mut attribute_map = HashMap::new();
        for item in data.split(',') {
            if let Some((key, value)) = M3U8::get_key_value_pair(item) {
                attribute_map.insert(key.to_string(), value.to_string());
            }
        }
        attribute_map
    }

    fn by_value(line: &str) -> (&str, &str) {
        let mut attribute = line.split(':');
        let tag = attribute.next().unwrap_or(&"");
        let data = attribute.next().unwrap_or(&"");
        (tag, data)
    }

    fn parse(&mut self, lines: &[String]) {
        let mut iter_lines = lines.iter();
        while let Some(line) = iter_lines.next() {
            let tag: Vec<&str> = line.split(':').collect();
            match tag.first() {
                Some(&"#EXTM3U") => (),
                Some(&"#EXT-X-INDEPENDENT-SEGMENTS") => {
                    self.independent_segments = true;
                },
                Some(&"EXT-X-VERSION") => {
                    let(_, data) = M3U8::by_value(line);
                    self.version = data.to_string();
                },
                Some(&"#EXT-X-MEDIA") => {
                    let(_, data) = M3U8::by_value(line);
                    let attributes = self.by_attribute(data);
                    self.media_tags.push(attributes);
                }
                Some(&"#EXT-X-I-FRAME-STREAM-INF") => {
                    let(_, data) = M3U8::by_value(line);
                    let attributes = self.by_attribute(data);
                    self.media_resources.push(attributes);
                },
                Some(&"#EXT-X-STREAM-INF") => {
                    let(_, data) = M3U8::by_value(line);
                    let mut attributes = self.by_attribute(data);
                    let uri = iter_lines.next().unwrap_or(&"".to_string()).to_string();
                    attributes.insert("uri".to_string(), uri);
                    self.variant_streams.push(attributes);
                },
                // Todo, Add Full Implementation
                _ => {
                    println!("Unhandled: {}", line);
                }
            }
        }
    }

    pub fn from_uri(uri: &str) -> Result<M3U8, ParseError> {
        let respose = reqwest::blocking::get(uri)?;
        let body = respose.text()?;
        let lines: Vec<String> = body
            .lines()
            .map(|m| m.to_string())
            .filter(|m| !m.is_empty())
            .collect();
        M3U8::validate(&lines)?;
        let mut m3u8 = M3U8::new();
        m3u8.parse(&lines);
        Ok(m3u8)
    }
}

#[cfg(test)]
mod tests {

    use crate::M3U8;

    #[test]
    fn it_works() {
        let uri =
        "https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8";

        match M3U8::from_uri(uri) {
            Ok(parsed) => {
                println!("{:?}", parsed);
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
        assert!(false);
    }
}