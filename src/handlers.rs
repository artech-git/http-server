use std::{ffi::OsString, path::PathBuf};

use http::{header::CONTENT_TYPE, HeaderValue};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream, BufWriter},
    net::TcpStream,
};

use crate::objects::create_data_response;

pub fn response_raw_vec(mut response: http::Response<Vec<u8>>) -> String {
    // Extract the status line
    let status_line = format!(
        "{:?} {} {}\r\n",
        response.version(),
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("")
    );

    // let value = response.headers_mut().entry(
    //     CONTENT_TYPE,
    // ).or_insert(HeaderValue::from_static("application/octet-stream"));
    
    if let None = response.headers_mut().get(CONTENT_TYPE) { 
        response.headers_mut().insert(CONTENT_TYPE,HeaderValue::from_static("text/plain"));
    
    }
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

    // response
    //     .headers_mut()
    //     .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

    if let None = response.headers_mut().get(CONTENT_TYPE) { 
        response.headers_mut().insert(CONTENT_TYPE,HeaderValue::from_static("text/plain"));
    
    }

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

pub async fn write_buffer_to_file(mut path: PathBuf, filename: PathBuf, mut buffer: Vec<u8>) -> Result<(), ()>{ 
    
    path.push(filename); 

    let file = match tokio::fs::File::create(path).await {
        Ok(val) => val , 
        Err(_) => { 
            return Err(());
        }
    };

    let mut buffered_file = BufWriter::new(file); 
    let _res = buffered_file.write_all(&buffer).await; 
    let _flushed = buffered_file.flush().await; 
    //TODO: just pray to god this takes place correctly

    if _res.is_err() || _flushed.is_err() { 
        return Err(()); 
    }

    return Ok(()); 
}

pub async fn convert_stream_to_lines(mut stream: &mut BufStream<TcpStream>) -> Vec<String> {
    // let mut vec_string = vec![];
    // let mut buf = "".to_string();

    // while let result = stream.read_line(&mut buf).await {
    //     if result.is_err() { 
    //         println!("err");
    //     }
    //     //TODO: change this to even parse and read the body content of a Request
    //     println!("buf: {:?}", buf); 
    //     // if val == 0 {
    //     //     break;
    //     // }
    //     vec_string.push(buf.clone());
    //     buf.clear();
    // }

    let stream = stream.get_ref(); 
    // let mut absolute_buffer = vec![]; 
    let mut large_string = "".to_string(); 

    loop {
        // Wait for the socket to be readable
        stream.readable().await.unwrap();

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
        let mut buf = vec![0; 1024];

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                // absolute_buffer.extend_from_slice(&buf); 
                buf.retain(|val| val != &('\0' as u8) );
                large_string.push_str(&String::from_utf8_lossy(&buf));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("blocked");
                break;
            }
            Err(e) => {
                // return Err(e.into());
                break; 
            }
        }
    }

    let mut vec_string = vec![]; 

    // Split the Request part and body part seperately ! this will create two longs strings ! 
    let mut request_body_splitted = large_string.split("\r\n\r\n")
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
    
    //push body first ! 😅
    let body = request_body_splitted.pop().unwrap(); 

    // further divide the request part into lines based upon escape characters 
    let mut splitted_request_lines = request_body_splitted[0]
        .split("\r\n")
        .map(|val| val.to_string())
        .collect::<Vec<String>>(); 

    splitted_request_lines.push("".to_string()); 

    splitted_request_lines.push(body); 
    
    vec_string = splitted_request_lines;

    println!("vec strings: {:#?}", vec_string); 
    vec_string
}
