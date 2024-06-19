use std::{ffi::OsString, path::PathBuf};

use http::{header::CONTENT_TYPE, HeaderValue};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufStream},
    net::TcpStream,
};

pub fn response_raw_vec(mut response: http::Response<Vec<u8>>) -> String {
    // Extract the status line
    let status_line = format!(
        "{:?} {} {}\r\n",
        response.version(),
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("")
    );

    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    // Extract headers
    let headers = response
        .headers()
        .iter()
        .map(|(key, value)| format!("{}: {}\r\n", key, value.to_str().unwrap_or("")))
        .collect::<String>();

    // Extract the body
    let body = response.body();

    // Combine the status line, headers, and body into a raw HTTP response string
    format!(
        "{}{}Content-Length: {}\r\n\r\n{}",
        status_line,
        headers,
        body.len(),
        std::str::from_utf8(body).unwrap()
    )
}

pub fn response_to_raw_string(mut response: http::Response<String>) -> String {
    // Extract the status line
    let status_line = format!(
        "{:?} {} {}\r\n",
        response.version(),
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("")
    );

    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    // Extract headers
    let headers = response
        .headers()
        .iter()
        .map(|(key, value)| format!("{}: {}\r\n", key, value.to_str().unwrap_or("")))
        .collect::<String>();

    // Extract the body
    let body = response.body();

    // Combine the status line, headers, and body into a raw HTTP response string
    format!(
        "{}{}Content-Length: {}\r\n\r\n{}\n",
        status_line,
        headers,
        body.len(),
        body
    )
}

pub fn get_path_name(val: String) -> String {
    let file_name = val
        .rsplitn(2, "/")
        .map(String::from)
        .collect::<Vec<String>>()
        .remove(0);
    file_name
}

pub async fn search_file_path(
    dir: impl Into<PathBuf>,
    filename: impl Into<OsString>,
) -> Result<PathBuf, ()> {
    let mut dir_entry = tokio::fs::read_dir(&(dir.into())).await.unwrap();
    let mut filename = filename.into();
    while let Ok(Some(val)) = dir_entry.next_entry().await {
        println!("val: {:?}", val.path());

        if val.file_name() == filename {
            return Ok(val.path().to_path_buf());
        }
    }

    return Err(());
}

pub async fn read_file_to_buffer(path: PathBuf) -> Result<Vec<u8>, ()> {
    let mut file = tokio::fs::File::open(path).await;
    let mut file_buffer = vec![];

    match file {
        Ok(mut file_open) => {
            println!("File found: ");

            let _res = file_open.read_to_end(&mut file_buffer).await;
            return Ok(file_buffer);
        }
        Err(err) => {
            println!("File not found");
            return Err(());
        }
    }
}

pub async fn process_buffer_to_response_buffer(buffer: Vec<u8>) -> Vec<u8> {
    let mut data_body = http::Response::default();
    *data_body.body_mut() = buffer;
    let parsed_bytes = response_raw_vec(data_body).as_bytes().to_vec();

    parsed_bytes
}

pub async fn convert_stream_to_lines(mut stream: &mut BufStream<TcpStream>) -> Vec<String> {
    let mut vec_string = vec![];
    let mut buf = "".to_string();

    while let Ok(_val) = stream.read_line(&mut buf).await {
        //TODO: change this to even parse and read the body content of a Request
        if buf.as_bytes() == b"\r\n" {
            break;
        }
        vec_string.push(buf.clone());
        buf.clear();
    }

    vec_string
}
