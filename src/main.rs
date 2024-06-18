// Uncomment this block to pass the first stage
use std::{
    fmt::format,
    // io::{BufRead, BufReader, Read, Write},
    // net::{Shutdown, TcpListener},
    str::FromStr,
    time::Duration,
};

use http::{header::CONTENT_TYPE, HeaderName, HeaderValue};
// use nom::AsBytes;
use objects::{HeaderMap, Request};
use tokio::{io::BufReader, net::TcpListener};
mod error;
mod objects;

use tokio::io::{AsyncBufRead, AsyncWriteExt, BufStream}; 
use tokio::io::AsyncBufReadExt; 

// fn main() {
//     // You can use print statements as follows for debugging, they'll be visible when running tests.
//     println!("Logs from your program will appear here!");

//     let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

//     let mut buf = String::with_capacity(1024);
//     let mut byte_buf: Vec<u8> = Vec::with_capacity(1024);

//     'listener: for stream in listener.incoming() {
//         byte_buf.clear();
//         buf.clear();

//         let mut data_response = http::Response::new(" ".to_string());
//         let err_response = http::Response::builder()
//             .status(404)
//             .body("Invalid request".to_string())
//             .unwrap();

//         match stream {
//             Ok(mut stream) => {
//                 // let _res = stream.set_nonblocking(true).unwrap();
//                 println!("accepted new connection");

//                 let mut buffered_stream = BufReader::new(&stream);

//                 let mut data_ingest: Vec<String> = buffered_stream
//                     .lines()
//                     .map(|line| line.unwrap())
//                     .take_while(|line| !line.is_empty())
//                     .collect();

//                 println!("moving forward: {:?}", buf);

//                 let (mut request_obj, mut header_obj) =
//                     match split_request_headers_data(data_ingest) {
//                         Ok((req, header)) => {
//                             println!("connection succeded");
//                             println!("request: {:?} \n header: {:#?}", req, header);
//                             (req, header)
//                         }
//                         Err(err) => {
//                             let err = stream
//                                 .write(response_to_raw_string(err_response.clone()).as_bytes())
//                                 .unwrap();
//                             continue 'listener;
//                         }
//                     };

//                 let path = request_obj.resource_path().to_string();
//                 match path {
//                     val if val == "/index.html".to_string() || val == "/".to_string() => {
//                         println!("path accepted");
//                         let resp_data = response_to_raw_string(data_response.clone());
//                         let err = stream.write(resp_data.as_bytes()).unwrap();
//                     }

//                     val if val.contains("/echo") => {
//                         println!("echo accepted");
//                         let mut resp_body = val
//                             .rsplitn(2, "/")
//                             .map(String::from)
//                             .collect::<Vec<String>>()
//                             .remove(0);

//                         *data_response.body_mut() = resp_body;

//                         let resp_data = response_to_raw_string(data_response.clone());

//                         let err = stream.write(resp_data.as_bytes()).unwrap();
//                     }

//                     val if val.contains("/user-agent") => {
//                         println!(" user agent accepted");

//                         let header_value = header_obj.remove(&objects::HeaderName::USER_AGENT);

//                         let header_string =
//                             if let Some(objects::HeaderValue::string(val)) = header_value {
//                                 val
//                             } else {
//                                 "none".to_string()
//                             };

//                         *data_response.body_mut() = header_string; 

//                         let resp_data = response_to_raw_string(data_response.clone());
//                         let err = stream.write(resp_data.as_bytes()).unwrap();
//                     }
//                     _ => {
//                         println!("invalid path");
//                         let err = stream
//                             .write(
//                                 response_to_raw_string(err_response.clone())
//                                     .to_string()
//                                     .as_bytes(),
//                             )
//                             .unwrap();
//                     }
//                 }
//             }

//             Err(e) => {
//                 println!("error: {}", e);
//             }
//         }
//     }
// }

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

    'listener: while let Ok((tcpstream, addr)) = listener.accept().await { 
        println!(" Thread handle : {counter}");
        counter += 1; 
        let (mut data_response, mut err_response) = (create_data_response(), create_err_response()); 

        let _handle = tokio::spawn(async move { 
            
            let mut buffered_stream = BufStream::new(tcpstream); 

            let mut data_ingest = {

                let mut vec_string = vec![]; 
                let mut line_data = &mut buffered_stream; 
                let mut buf = "".to_string(); 

                while let Ok(_val) = line_data.read_line(&mut buf).await { 
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