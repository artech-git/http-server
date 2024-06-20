use http::{header::{ACCEPT_ENCODING, CONTENT_ENCODING}, HeaderMap, HeaderName, HeaderValue, Uri};
use std::{collections::HashMap, str::FromStr};
use string_enum::StringEnum;

use crate::error::{HeaderError, HttpRequestError};

#[derive(Debug)]
pub struct HttpRequest {
    pub request: Request,
    pub headers: HeaderMap,
    pub body: String
}

impl HttpRequest {
    pub fn get_req_ref(&self) -> &Request {
        &self.request
    }

    pub fn get_headers_ref(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn get_headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    pub fn get_req_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    pub fn get_body_content(&self) -> &String {
        &self.body
    }

    pub fn from_string_line_collection(
        mut data: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut data_iterator = data.into_iter();

        let request_data = data_iterator
            .next()
            .ok_or("Request data not found".to_string())?;

        println!("1: {:?}", request_data);

        let request = Request::from_str(&request_data).map_err(|val| format!("{:?}", val))?;

        let mut headermap = HeaderMap::default();
        let mut body_data = "".to_string(); 

        while let Some(line) = (&mut data_iterator).next() {

            if line.is_empty() { 
                body_data = (&mut data_iterator).next().unwrap(); 
                continue;     
            }

            let mut splitter = line.splitn(2, ":");
            let header = splitter.next().ok_or(HeaderError::InvalidHeaderName)?;
            let value = splitter
                .next()
                .ok_or(HeaderError::InvalidHeaderValue)?
                .trim();
            let _y = headermap.insert(
                HeaderName::from_str(header).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }

        Ok(Self {
            request: request,
            headers: headermap,
            body: body_data
        })
    }
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    uri: Uri,
    version: HTTPVersion,
}

impl Request {
    pub fn get_method(&self) -> &Method {
        &self.method
    }
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

#[derive(StringEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
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

pub fn create_data_response(req: &HeaderMap) -> http::Response<Vec<u8>> {

    let mut res = http::Response::new(vec![]);

    if let Some(val) = req.get(ACCEPT_ENCODING) {
        if let Some(encoding) = ServerEncoding::get_header_encoding(val) {
            res.headers_mut().insert(CONTENT_ENCODING, encoding.into_header());
        }
    }
    res
}

pub fn create_err_response(req: &HeaderMap) -> http::Response<Vec<u8>> {
    // http::Response::new(" ".to_string())
    let mut err_response = http::Response::builder()
        .status(404)
        .body("Invalid request".as_bytes().to_vec())
        .unwrap();
    if let Some(val) = req.get(ACCEPT_ENCODING) {
        if let Some(encoding) = ServerEncoding::get_header_encoding(val) {
            err_response.headers_mut().insert(CONTENT_ENCODING, encoding.into_header());
        }
    }
    err_response
}

#[derive(StringEnum )]
pub enum ServerEncoding {
    /// `gzip`
    GZIP, 
    // Invalid 
}

// type EncoderForamt = impl FnOnce(Vec<u8>) -> Vec<u8>;

impl ServerEncoding { 
    pub fn return_encoder(&self, buffer: Vec<u8>) -> Vec<u8> { 
        todo!()
    }
    pub fn get_header_encoding(headervalue: &HeaderValue) -> Option<Self> { 
        let header_name_string = headervalue.to_str().unwrap(); 
        if let Ok(val) = Self::from_str(&header_name_string) { 
            return Some(val); 
        }
        return None; 
    }
    pub fn into_header(self) -> HeaderValue { 
        HeaderValue::from_static(self.as_str())
    }
}