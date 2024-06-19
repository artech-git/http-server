use std::ffi::OsString;
use std::path::PathBuf;
// Uncomment this block to pass the first stage
use std::{
    fmt::format,
    // io::{BufRead, BufReader, Read, Write},
    // net::{Shutdown, TcpListener},
    str::FromStr,
    time::Duration,
};

use clap::{arg, value_parser, Arg, Command};
use http::{header::CONTENT_TYPE, HeaderName, HeaderValue};
// use nom::AsBytes;
use objects::{HeaderMap, Request};
use tokio::{io::BufReader, net::TcpListener};
mod error;
mod objects;

use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufStream}; 
use tokio::io::AsyncBufReadExt; 

fn split_request_headers_data(mut data: Vec<String>) -> Result<(Request, HeaderMap), String> {
    // let mut data_iterator = data.splitn(2, std::str::from_utf8(b"\r\n").unwrap());
    let mut data_iterator = data.into_iter();

    let request_data = data_iterator
        .next()
        .ok_or("Request data not found".to_string())?;
    println!("1: {:?}", request_data);
    // let header_data = data_iterator.next().ok_or("Header data not found".to_string())?;

    let request = Request::from_str(&request_data).map_err(|val| format!("{:?}", val))?;
    // let method_data = HeaderMap::from_str(header_data).map_err(|val| format!("{:?}", val))?;
    let mut method_data = HeaderMap::default();

    for line in data_iterator {
        let _y = method_data.from_line(line);
    }

    Ok((request, method_data))
}

fn response_raw_vec(mut response: http::Response<Vec<u8>>) -> String {
    // Extract the status line
    let status_line = format!(
        "{:?} {} {}\r\n",
        response.version(),
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("")
    );

    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
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

fn response_to_raw_string(mut response: http::Response<String>) -> String {
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

#[tokio::main]
async fn main() {

    let mut listener = TcpListener::bind("127.0.0.1:4221").await.unwrap(); 
    let mut counter = 0; 

    let matches = Command::new("Async File Reader")
        .version("1.0")
        .author("Meow meow@meow.com")
        .about("Reads a file asynchronously using Tokio")
        .arg(
            arg!(
                -d --directory <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .get_matches(); 

    // Get the file path from the command-line arguments
    let dir_string = matches.get_one::<PathBuf>("directory").unwrap_or(&PathBuf::from("./")).to_owned();

    println!("dir: {:?}", dir_string);

    let curr_dir = PathBuf::from(dir_string);  

    'listener: while let Ok((tcpstream, addr)) = listener.accept().await { 

        println!(" Thread handle : {counter}");

        counter += 1; 
        let cloned_curr_dir = curr_dir.clone(); 

        let (mut data_response, mut err_response) = (create_data_response(), create_err_response()); 

        let _handle = tokio::spawn(async move { 
            
            let mut buffered_stream = BufStream::new(tcpstream); 

            let mut data_ingest = {

                let mut vec_string = vec![]; 
                let mut line_data = &mut buffered_stream; 
                let mut buf = "".to_string(); 

                while let Ok(_val) = line_data.read_line(&mut buf).await { 
                    //TODO: change this to even parse and read the body content of a Request
                    if buf.as_bytes() == b"\r\n" {
                        break; 
                    }
                    vec_string.push(buf.clone()); 
                    buf.clear(); 
                }
                vec_string
            };

            let (mut request_obj, mut header_obj) = match split_request_headers_data(data_ingest) {
                Ok((req, header)) => {
                    println!("connection succeded");
                    println!("request: {:?} \n header: {:#?}", req, header);
                    (req, header)
                }
                Err(err) => {
                    let err = buffered_stream
                        .write(response_to_raw_string(err_response.clone()).as_bytes())
                        .await
                        .unwrap();
                    return;
                }
            };

            let path = request_obj.resource_path().to_string();

            match path {
                val if val.contains("/file") => { 
                    println!("file accepted");

                    let mut file_name = val
                        .rsplitn(2, "/")
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .remove(0);

                    let mut file_name = OsString::from(file_name); 

                    let mut buffer = {

                        // let mut path_buf = PathBuf::from_str(&curr_dir).unwrap(); 
                        // let mut path_buf = (&curr_dir).clone(); 
                        let mut dir_entry = tokio::fs::read_dir(&(cloned_curr_dir)).await.unwrap(); 

                        let mut file_buffer = vec![];
                        let mut returning_bytes = vec![]; 

                        let mut file_not_found = true; 
                        while let Ok(Some(val)) = dir_entry.next_entry().await { 

                            println!(" val: {:?}", val.path()); 

                            if val.file_name() == file_name {

                                file_not_found = false; 
                                let mut file = tokio::fs::File::open(val.path()).await;

                                match file { 

                                    Ok(mut file_open) => { 

                                        println!("File found: "); 

                                        let _res = file_open.read_to_end(&mut file_buffer).await; 
                                        let mut data_body = http::Response::default(); 
                                        *data_body.body_mut() = file_buffer; 

                                        let bytes = response_raw_vec(data_body);
                                        returning_bytes = bytes.as_bytes().to_vec(); 
                                        // let err = buffered_stream.write(resp_data.as_bytes()).await.unwrap();
                                        break ;  
                                    }
                                    Err(err) => { 
                                        println!("File not found");
                                        returning_bytes = response_to_raw_string(err_response.clone())
                                                .to_string()
                                                .as_bytes()
                                                .to_vec();
                                        break ;
                                    }
                                }

                            }
                        }

                        if file_not_found { 
                            println!("File not discovered");
                            returning_bytes = response_to_raw_string(err_response.clone())
                                    .as_bytes()
                                    .to_vec();
                        }
                        returning_bytes
                    };

                let err = buffered_stream.write(&buffer).await.unwrap();

                }
                val if val == "/index.html".to_string() || val == "/".to_string() => {
                    println!("path accepted");
                    let resp_data = response_to_raw_string(data_response.clone());
                    let err = buffered_stream.write(resp_data.as_bytes()).await.unwrap();
                }
                val if val.contains("/echo") => {
                    println!("echo accepted");

                    let mut resp_body = val
                        .rsplitn(2, "/")
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .remove(0);

                    *data_response.body_mut() = resp_body;

                    let resp_data = response_to_raw_string(data_response.clone());

                    let err = buffered_stream.write(resp_data.as_bytes()).await.unwrap();
                }
                val if val.contains("/user-agent") => {
                    println!(" user agent accepted");

                    let header_value = header_obj.remove(&objects::HeaderName::USER_AGENT);

                    let header_string =
                        if let Some(objects::HeaderValue::string(val)) = header_value {
                            val
                        } else {
                            "none".to_string()
                        };

                    *data_response.body_mut() = header_string; 

                    let resp_data = response_to_raw_string(data_response.clone());
                    let err = buffered_stream.write(resp_data.as_bytes()).await.unwrap();
                }
                _ => {
                    println!("invalid path");
                    let err = buffered_stream
                        .write(
                            response_to_raw_string(err_response.clone())
                                .to_string()
                                .as_bytes(),
                        )
                        .await
                        .unwrap();
                }
            }
            let _t = buffered_stream.flush().await; 
        }

        );


    }
}

fn create_data_response() -> http::Response<String> { 
    http::Response::new(" ".to_string())
}

fn create_err_response() -> http::Response<String> { 
    // http::Response::new(" ".to_string())
    let err_response = http::Response::builder()
            .status(404)
            .body("Invalid request".to_string())
            .unwrap();
    err_response
}