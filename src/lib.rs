//! A library to parse m3u8 playlist.
//!
//! # Examples
//!
//! let uri = "http://<domain>/path/playlist.m3u8"
//! let parsed_m3u8 = M3U8::from_uri(uri).unwrap();
//!
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

const EXTM3U: &str = "#EXTM3U";
const EXT_X_INDEPENDENT_SEGMENTS: &str = "#EXT-X-INDEPENDENT-SEGMENTS";
const EXT_X_VERSION: &str = "#EXT-X-VERSION";
const EXT_X_MEDIA: &str = "#EXT-X-MEDIA";
const EXT_X_I_FRAME_STREAM_INF: &str = "#EXT-X-I-FRAME-STREAM-INF";
const EXT_X_STREAM_INF: &str = "#EXT-X-STREAM-INF";

/// Error Wrapper for M3U8 Parsing
#[derive(Debug)]
pub enum ParseError {
    InvalidM3U8(String),
    ReqwestError(reqwest::Error),
}

/// Map a Reqwest Error to our Error Wrapper
impl From<reqwest::Error> for ParseError {
    fn from(err: reqwest::Error) -> ParseError {
        ParseError::ReqwestError(err)
    }
}

/// Represent M3U8 tag types
#[derive(Debug, PartialEq)]
enum TagTypes {
    ExtM3U,
    ExtXIndependentSegments,
    ExtXVersion,
    ExtXMedia,
    ExtXIFrameStreamInf,
    ExtXStreamInf,
}

/// Tag types fromStr
impl FromStr for TagTypes {
    type Err = ();
    fn from_str(input: &str) -> Result<TagTypes, Self::Err> {
        match input {
            EXTM3U => Ok(TagTypes::ExtM3U),
            EXT_X_INDEPENDENT_SEGMENTS => Ok(TagTypes::ExtXIndependentSegments),
            EXT_X_VERSION => Ok(TagTypes::ExtXVersion),
            EXT_X_MEDIA => Ok(TagTypes::ExtXMedia),
            EXT_X_I_FRAME_STREAM_INF => Ok(TagTypes::ExtXIFrameStreamInf),
            EXT_X_STREAM_INF => Ok(TagTypes::ExtXStreamInf),
            _ => Err(()),
        }
    }
}

/// TagTypes as a Display type for string formatting
impl fmt::Display for TagTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TagTypes::ExtM3U => write!(f, "{}", EXTM3U),
            TagTypes::ExtXIndependentSegments => write!(f, "{}", EXT_X_INDEPENDENT_SEGMENTS),
            TagTypes::ExtXVersion => write!(f, "{}", EXT_X_VERSION),
            TagTypes::ExtXMedia => write!(f, "{}", EXT_X_MEDIA),
            TagTypes::ExtXIFrameStreamInf => write!(f, "{}", EXT_X_I_FRAME_STREAM_INF),
            TagTypes::ExtXStreamInf => write!(f, "{}", EXT_X_STREAM_INF),
        }
    }
}

/// Our Parsed M3U8
#[derive(Default, Debug)]
pub struct M3U8 {
    independent_segments: bool,
    version: String,
    media_tags: Vec<HashMap<String, String>>,
    variant_streams: Vec<HashMap<String, String>>,
    media_resources: Vec<HashMap<String, String>>,
}

/// Implementation for M3U8
impl M3U8 {
    /// New sets default version to 2 (Lowest Protocol) and returns M3U8
    fn new() -> Self {
        M3U8 {
            version: "2".to_string(), // Default if no version supplied
            ..Default::default()
        }
    }

    /// Validates our data
    fn validate(lines: &[String]) -> Result<(), ParseError> {
        let intro = lines
            .first()
            .ok_or_else(|| ParseError::InvalidM3U8("Invalid M3U8 format".to_string()))?;

        // If no ExtM3U
        if TagTypes::from_str(intro) != Ok(TagTypes::ExtM3U) {
            return Err(ParseError::InvalidM3U8("Missing #EXTM3U".to_string()));
        }

        // If Multiple Versions per RFC
        if lines
            .iter()
            .filter(|&n| TagTypes::from_str(n) == Ok(TagTypes::ExtXVersion))
            .count()
            > 1
        {
            return Err(ParseError::InvalidM3U8(
                "Invalid M3U8, multiple version tags found.".to_string(),
            ));
        }
        Ok(())
    }

    /// Parses single KEY=VALUE line
    fn get_key_value_pair(item: &str) -> Option<(String, String)> {
        let mut attr = item.split('=');
        let key = match attr.next() {
            Some(key) => key.to_string(),
            None => return None,
        };
        let value = match attr.next() {
            Some(value) => value.replace(&['\"', '\''][..], ""), // Replace escape chars
            None => return None,
        };
        Some((key, value))
    }

    /// Parses all attribute lines containing KEY=VALUE
    fn by_attribute(&mut self, data: &str) -> HashMap<String, String> {
        let mut attribute_map = HashMap::new();
        for item in data.split(',') {
            if let Some((key, value)) = M3U8::get_key_value_pair(item) {
                attribute_map.insert(key.to_string(), value.to_string());
            }
        }
        attribute_map
    }

    /// Parses simple key,value type
    fn by_value(line: &str) -> (&str, &str) {
        let mut attribute = line.split(':');
        let tag = attribute.next().unwrap_or("");
        let data = attribute.next().unwrap_or("");
        (tag, data)
    }

    /// Parse and match by our tag types
    fn parse(&mut self, lines: &[String]) {
        let mut iter_lines = lines.iter();
        while let Some(line) = iter_lines.next() {
            let tag: Vec<&str> = line.split(':').collect();
            let tag_type = if let Some(tag) = tag.first() {
                TagTypes::from_str(tag)
            } else {
                break;
            };
            match tag_type {
                Ok(TagTypes::ExtM3U) => (),
                Ok(TagTypes::ExtXIndependentSegments) => {
                    self.independent_segments = true;
                }
                Ok(TagTypes::ExtXVersion) => {
                    let (_, data) = M3U8::by_value(line);
                    self.version = data.to_string();
                }
                Ok(TagTypes::ExtXMedia) => {
                    let (_, data) = M3U8::by_value(line);
                    let attributes = self.by_attribute(data);
                    self.media_tags.push(attributes);
                }
                Ok(TagTypes::ExtXIFrameStreamInf) => {
                    let (_, data) = M3U8::by_value(line);
                    let attributes = self.by_attribute(data);
                    self.media_resources.push(attributes);
                }
                Ok(TagTypes::ExtXStreamInf) => {
                    let (_, data) = M3U8::by_value(line);
                    let mut attributes = self.by_attribute(data);
                    let uri = iter_lines.next().unwrap_or(&"".to_string()).to_string();
                    attributes.insert("uri".to_string(), uri);
                    self.variant_streams.push(attributes);
                }
                // Todo, Add Full Implementation
                _ => {
                    println!("Unhandled: {}", line);
                }
            }
        }
    }

    /// Used to sort Parsed Vectors
    fn sort_list_by_key(list: &mut Vec<HashMap<String, String>>, sort_by: &str) {
        list.sort_by(|a, b| {
            let item1 = match a.get(sort_by) {
                Some(item1) => item1,
                None => "",
            };
            let item2 = match b.get(sort_by) {
                Some(item1) => item1,
                None => "",
            };
            item1.cmp(item2)
        });
    }

    /// Returns Cloned Vec of media resources sorted by provided key
    pub fn get_media_resources(&mut self, sort_by: &str) -> Vec<HashMap<String, String>> {
        M3U8::sort_list_by_key(&mut self.media_resources, sort_by);
        self.media_resources.clone()
    }

    /// Returns Cloned Vec of media tags sorted by provided key
    pub fn get_media_tags(&mut self, sort_by: &str) -> Vec<HashMap<String, String>> {
        M3U8::sort_list_by_key(&mut self.media_tags, sort_by);
        self.media_tags.clone()
    }

    /// Returns Cloned Vec of variant streams sorted by provided key
    pub fn get_variant_streams(&mut self, sort_by: &str) -> Vec<HashMap<String, String>> {
        M3U8::sort_list_by_key(&mut self.variant_streams, sort_by);
        self.variant_streams.clone()
    }

    /// Takes URI return parsed M3U8 otherwise raises ParseError
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

    /// Process our example
    #[test]
    fn it_parses_example_uri() {
        let uri =
            "https://lw.bamgrid.com/2.0/hls/vod/bam/ms02/hls/dplus/bao/master_unenc_hdr10_all.m3u8";

        let result = M3U8::from_uri(uri);

        assert!(result.is_ok());

        let mut parsed = result.unwrap();

        assert_eq!(parsed.version, "2");
        assert_eq!(parsed.independent_segments, true);
        assert_eq!(parsed.media_tags.len(), 4);
        assert_eq!(
            parsed.media_tags.first().unwrap().get("TYPE"),
            Some(&"AUDIO".to_string())
        );
        assert_eq!(
            parsed.media_tags.first().unwrap().get("URI"),
            Some(&"audio/unenc/aac_128k/vod.m3u8".to_string())
        );
        assert_eq!(parsed.media_resources.len(), 2);
        assert_eq!(
            parsed.media_resources.first().unwrap().get("URI"),
            Some(&"hdr10/unenc/3300k/vod-iframe.m3u8".to_string())
        );
        assert_eq!(parsed.media_tags.len(), 4);
        assert_eq!(
            parsed.media_tags.first().unwrap().get("URI"),
            Some(&"audio/unenc/aac_128k/vod.m3u8".to_string())
        );

        // Test fetch and sorting media resources
        let media_resources = parsed.get_media_resources("BANDWIDTH");
        assert_eq!(media_resources.first().unwrap()["BANDWIDTH"], "222552");
        assert_eq!(media_resources.last().unwrap()["BANDWIDTH"], "77758");
        // Reverse and still see if in order
        parsed.media_resources.reverse();
        let media_resources = parsed.get_media_resources("BANDWIDTH");
        assert_eq!(media_resources.first().unwrap()["BANDWIDTH"], "222552");
        assert_eq!(media_resources.last().unwrap()["BANDWIDTH"], "77758");

        // Test fetch and sorting media streams
        parsed.media_tags.reverse();
        let media_tags = parsed.get_media_tags("CHANNELS");
        assert_eq!(media_tags.first().unwrap()["CHANNELS"], "16/JOC");
        assert_eq!(media_tags.last().unwrap()["CHANNELS"], "6");

        // Test fetch and sorting variant streams
        let variant_streams = parsed.get_variant_streams("BANDWIDTH");
        assert_eq!(variant_streams.first().unwrap()["BANDWIDTH"], "10429877");
        assert_eq!(variant_streams.last().unwrap()["BANDWIDTH"], "9661857");
    }

    #[test]
    /// Tests Invalid bad uri fails
    /// Todo: Assert specific ErrorType
    fn it_fails_invalid_uri() {
        let m3u8_result = M3U8::from_uri("bad");
        assert!(m3u8_result.is_err());
    }

    #[test]
    /// Tests Invalid m3u8 fails
    /// Todo: Assert specific ErrorType
    fn it_fails_invalid_m3u8() {
        let m3u8_result = M3U8::from_uri("www.example.com");
        assert!(m3u8_result.is_err());
    }
}
