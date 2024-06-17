use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum HttpRequestError{
//     #[error(transparent)]
//     InvalidRequest(RequestError),
//     #[error(transparent)]
//     InvalidHeaders(HeaderError)
// }

#[derive(Debug, Error)]
pub enum HttpRequestError {
    #[error("Invalid Method")]
    InvalidMethod,
    #[error("Invalid Target path")]
    InvalidTarget,
    #[error("Invalid HTTP Version")]
    InvalidVersion,
    #[error(transparent)]
    HeaderMapError(HeaderError),
}

#[derive(Debug, Error)]
pub enum HeaderError {
    #[error("Error: Invalid HeaderName")]
    InvalidHeaderName,
    #[error("Error: Invalid HeaderValue")]
    InvalidHeaderValue,
}
