use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{arg, value_parser, Command};
use handlers::{
    convert_stream_to_lines, get_path_name, read_file_to_buffer,
    response_raw_vec, search_file_path, write_buffer_to_file,
};
use http::{HeaderValue, StatusCode};
use objects::{create_data_response, create_err_response, HttpRequest};
use tokio::net::TcpListener;
mod error;
mod handlers;
mod objects;

use tokio::io::{AsyncWriteExt, BufStream};

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
    let curr_dir = matches
        .get_one::<PathBuf>("directory")
        .unwrap_or(&PathBuf::from("./"))
        .to_owned();
    //println!("dir: {:?}", curr_dir);

    while let Ok((tcpstream, addr)) = listener.accept().await {
        //println!(" Thread handle : {counter}");
        counter += 1;

        let cloned_curr_dir = (&curr_dir).clone();

        let _handle = tokio::spawn(async move {
            let mut buffered_stream = BufStream::new(tcpstream);

            let data_ingest = convert_stream_to_lines(&mut buffered_stream).await;

            //println!("Raw data: {:?}", data_ingest);

            let mut request_object = match HttpRequest::from_string_line_collection(data_ingest) {
                Ok(req_obj) => {
                    //println!("request parsed into lines");
                    //println!(
                        // "request: {:?} \n headers: {:#?}",
                        // req_obj.get_req_ref(),
                        // req_obj.get_headers_ref()
                    // );
                    req_obj
                }
                Err(err) => {
                    // let err = buffered_stream
                    //     .write(response_to_raw_string(create_err_response()).as_bytes())
                    //     .await
                    //     .unwrap();
                    return;
                }
            };

            let path = request_object.get_req_ref().resource_path().to_string();

            match path {
                
                val if (val.contains("/file") && *request_object.get_req_ref().get_method() == objects::Method::POST) => { 
                    //println!("upload req accepted");

                    let file_name = get_path_name(val); 

                    let buffer_content = request_object.get_body_content().as_bytes().to_vec(); 
                    let res = write_buffer_to_file(cloned_curr_dir, PathBuf::from(file_name), buffer_content).await; 

                    let response = if let Ok(_) = res { 
                        let mut res = create_data_response(request_object.get_headers_ref());
                        *res.status_mut() = StatusCode::from_u16(201).unwrap(); 
                        res
                    }
                    else { 
                        create_err_response(request_object.get_headers_ref())
                    };

                    let buffer  = response_raw_vec(response); 
                    let err = buffered_stream.write(&buffer).await.unwrap();
                }

                val if (val.contains("/file") && *request_object.get_req_ref().get_method() == objects::Method::GET) => {
                    //println!("file accepted");

                    let file_name = get_path_name(val);

                    let mut file_name = OsString::from(file_name);

                    let mut buffer = if let Ok(val) =
                        search_file_path(cloned_curr_dir, file_name).await
                    {
                        match read_file_to_buffer(val).await {
                            Ok(val) => {
                                let mut data_body = http::Response::default();
                                *data_body.body_mut() = val;
                                data_body.headers_mut().insert(http::header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
                                let parsed_bytes = response_raw_vec(data_body);

                                parsed_bytes
                            }
                            Err(err) => {
                                let err_response = create_err_response(request_object.get_headers_ref());
                                response_raw_vec(err_response)
                            }
                        }
                    } else {
                        let err_response = create_err_response(request_object.get_headers_ref());
                        response_raw_vec(err_response)
                    };

                    let err = buffered_stream.write(&buffer).await.unwrap();
                }
                val if val == "/index.html".to_string() || val == "/".to_string() => {
                    //println!("path accepted");
                    let resp_data = response_raw_vec(create_data_response(request_object.get_headers_ref()));
                    let err = buffered_stream.write(&resp_data).await.unwrap();
                }
                val if val.contains("/echo") => {
                    //println!("echo accepted");

                    let mut resp_body = get_path_name(val).as_bytes().to_vec();
                    // let data_response = process_buffer_to_response_buffer(resp_body).await;
                    let data_response = {
                        let mut data_body = create_data_response(request_object.get_headers_ref());
                        *data_body.body_mut() = resp_body;
                        let parsed_bytes = response_raw_vec(data_body);

                        parsed_bytes
                    };

                    let err = buffered_stream.write(&data_response).await.unwrap();
                }
                val if val.contains("/user-agent") => {
                    //println!(" user agent accepted");

                    //TODO: ensure this user-agent header parsing is correct ?
                    let user_agent_header = http::HeaderName::from_str("user-agent").unwrap();

                    let header_value = request_object
                        .get_headers_mut()
                        .remove(user_agent_header)
                        .unwrap_or(HeaderValue::from_static("none"))
                        .to_str()
                        .unwrap()
                        .to_owned();

                    // let buffer =
                    //     process_buffer_to_response_buffer(header_value.as_bytes().to_vec()).await;

                    let buffer = { 
                        let mut data_body = create_data_response(request_object.get_headers_ref());
                        *data_body.body_mut() = header_value.as_bytes().to_vec();
                        let parsed_bytes = response_raw_vec(data_body);

                        parsed_bytes
                    };

                    let err = buffered_stream.write(&buffer).await.unwrap();
                }
                _ => {
                    //println!("invalid path");
                    let err_response = create_err_response(request_object.get_headers_ref());
                    let _err = buffered_stream
                        .write(&response_raw_vec(err_response))
                        .await
                        .unwrap();
                }
            }
            let _t = buffered_stream.flush().await;
        });
    }
}
