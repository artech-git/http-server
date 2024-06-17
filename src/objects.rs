use http::Uri;
use std::{collections::HashMap, str::FromStr};
use string_enum::StringEnum;

use crate::error::{HeaderError, HttpRequestError};

#[derive(Debug)]
pub struct HttpRequest {
    request: Request,
    headers: HeaderMap,
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    // target: String,
    uri: Uri,
    version: HTTPVersion,
}

impl Request {
    // pub fn compare_resource_path(&self, path: impl AsRef<str>) -> bool {
    //     self.target == path.as_ref()
    // }
    pub fn resource_path(&self) -> &str {
        self.uri.path()
    }
}

impl FromStr for Request {
    type Err = HttpRequestError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut broken_data = value.split(" ");
        let method_input = broken_data.next().ok_or(HttpRequestError::InvalidMethod)?;
        let method = Method::from_str(method_input).map_err(|v| HttpRequestError::InvalidMethod)?;
        let uri_string = broken_data
            .next()
            .ok_or(HttpRequestError::InvalidTarget)?
            .to_string();
        let uri = Uri::from_str(&uri_string).map_err(|_| HttpRequestError::InvalidTarget)?;
        let version_value = broken_data.next().ok_or(HttpRequestError::InvalidVersion)?;
        // let version = HTTPVersion::from_str(version_value).map_err(|v| HttpRequestError::InvalidVersion)?;
        let version = HTTPVersion::HTTP1_1; //TODO: change this assumption to actual request being made either http1.1 or http2
        Ok(Self {
            method,
            uri,
            version,
        })
    }
}

#[derive(StringEnum)]
enum Method {
    /// `GET`
    GET,
    /// `POST`
    POST,
    /// `PUT`
    PUT,
    /// `DELETE`
    DELETE,
}

#[derive(StringEnum)]
enum HTTPVersion {
    /// `HTTP1_1`
    HTTP1_1,
    /// `HTTP2`
    HTTP2,
}

// /===========================================
#[derive(Debug)]
pub struct HeaderMap {
    headerfields: HashMap<HeaderName, HeaderValue>,
}

impl std::ops::Deref for HeaderMap {
    type Target = HashMap<HeaderName, HeaderValue>;
    fn deref(&self) -> &Self::Target {
        &self.headerfields
    }
}

impl std::ops::DerefMut for HeaderMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.headerfields
    }
}

impl HeaderMap {
    pub fn from_line(&mut self, string_line: String) -> Result<(), HeaderError> {
        let mut splitter = string_line.splitn(2, ":");

        let header = splitter.next().ok_or(HeaderError::InvalidHeaderName)?;
        let value = splitter.next().ok_or(HeaderError::InvalidHeaderValue)?.trim();

        let headername =
            HeaderName::from_str(header).map_err(|val| HeaderError::InvalidHeaderName)?;
        let headervalue = HeaderValue::string(value.to_string());

        self.headerfields.insert(headername, headervalue);

        Ok(())
    }
}

impl std::default::Default for HeaderMap {
    fn default() -> Self {
        Self {
            headerfields: HashMap::new(),
        }
    }
}

impl FromStr for HeaderMap {
    type Err = HeaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hm = HashMap::new();
        let mut field_iter = s.split(std::str::from_utf8(b"\n").unwrap());

        for fields in field_iter {
            let (header, value) = {
                let mut splitter = fields.split(":");

                let header = if let Some(header) = splitter.next() {
                    // println!(" header: {:?}", header);
                    header
                } else {
                    continue;
                };

                let value = if let Some(value) = splitter.next() {
                    // println!("value: {:?}", value);
                    value
                } else {
                    continue;
                };

                (header, value)
            };

            let headername = HeaderName::from_str(header).unwrap();
            let headervalue = HeaderValue::string(value.to_string());

            hm.insert(headername, headervalue);
        }

        Ok(Self { headerfields: hm })
    }
}

#[derive(StringEnum, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeaderName {
    /// `CONTENT_TYPE`
    CONTENT_TYPE,
    /// `CONTENT_LENGTH`
    CONTENT_LENGTH,
    /// `Accept`
    ACCEPT,
    /// `User-Agent`
    USER_AGENT,
    /// `Host`
    HOST,
}

#[derive(Debug, PartialEq)]
pub enum HeaderValue {
    string(String),
    bytes(Vec<u8>),
}
