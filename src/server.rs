use std::convert::Infallible;
use std::net::{SocketAddr, IpAddr};
use hyper::{header, Body, Request, Response, Method, StatusCode};
use hyper::Server as Hyper_Server;
use hyper::service::{make_service_fn, service_fn};

use tokio::io::AsyncReadExt;
use tokio::fs::File;
use tokio::sync::mpsc::Sender;

use crate::sink::DataSink;
use crate::setup_response;

use serde::{Deserialize, Serialize};

use crate::manager;

#[derive(Serialize, Deserialize)]
struct JsonMessage<'a> {
    error: &'a str
}

pub struct Server {
    ip: IpAddr,
    port: u16,
    web_path: String,
    memory: DataSink,
}

impl Server {
    pub fn new(ip: IpAddr, port: u16, web_path: String, memory: DataSink) -> Self {
        Self {
            ip,
            port,
            web_path,
            memory
        }
    }
    pub async fn run(&self, tx: Sender<manager::Command>) {
        let addr = SocketAddr::new(self.ip, self.port);

        // A `Service` is needed for every connection, so this
        // creates one from our `hello_world` function.
        let make_svc = make_service_fn(move |_conn| {
            let context = tx.clone();
            let web_path = self.web_path.clone();
            let memory = self.memory.clone();
            // service_fn converts our function into a `Service`
            let service = service_fn(move |req| {
                //handle(context.clone(), addr, req)
                Server::serve_page(req, memory.clone(), context.clone(), web_path.clone())
            });
            async move { Ok::<_, Infallible>(service) }
        });

        let server = Hyper_Server::bind(&addr).serve(make_svc);

        println!("Listening on http://{}", addr);

        //Signal shutdown wurde entfernt weil er den anderen Thread dann nicht mehr killt
        //let shutdown = server.with_graceful_shutdown(Server::shutdown_signal());

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }

    }
    async fn serve_page(req: Request<Body>, data_memory: DataSink, tx: Sender<manager::Command>, web_path: String) -> Result<Response<Body>, Infallible> {
        let mut response = Response::new(Body::empty());
        let mut buffer = Vec::<u8>::new();
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") | (&Method::GET, "index.html") => {
                Server::load_data(&mut buffer, format!("{}/index.html", web_path)).await;
            },
            (&Method::GET, "/refresh") => {
                //wir löschen die daten aus dem speicher damit wir sehen, dass einmal alles neu geladen wird
                {
                    let init_memory = setup_response();
                    let mut memory = data_memory.lock().await;
                    *memory = vec![init_memory];
                }
                *response.status_mut() = StatusCode::OK;
                tx.send(manager::Command::Server("refresh")).await.unwrap();
            },
            (&Method::GET, "/getlist/all") => {
                let memory = data_memory.lock().await;
                *response.status_mut() = StatusCode::OK;
                let j = serde_json::to_string(&*memory).unwrap();
                response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                buffer = j.as_bytes().to_vec();
            },
            (&Method::GET, "/getdata") => {
                match req.uri().query() {
                    Some(query) => {
                        let query_parts: Vec<&str> = query.split('=').collect();
                        if query_parts[0] == "id" {
                            if let Ok(id) = query_parts[1].parse::<u32>() {
                                let mut found_id = false;
                                let memory = data_memory.lock().await;
                                for customer in &*memory {
                                    if id == customer.id {
                                        //Found Customer Id and returns the data
                                        *response.status_mut() = StatusCode::OK;
                                        let j = serde_json::to_string(&customer).unwrap();
                                        response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                                        buffer = j.as_bytes().to_vec();
                                        found_id = true;
                                        break;
                                    }
                                }
                                if !found_id {
                                    //Customer Id wasn't found in the memory
                                    let error = JsonMessage {error: "Customer Not Found"};
                                    *response.status_mut() = StatusCode::NOT_FOUND;
                                    let j = serde_json::to_string(&error).unwrap();
                                    response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                                    buffer = j.as_bytes().to_vec();
                                }
                            } else {
                                //Wrong datatype was entered as a id
                                let error = JsonMessage {error: "Id must be an integer"};
                                *response.status_mut() = StatusCode::NOT_FOUND;
                                let j = serde_json::to_string(&error).unwrap();
                                response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                                buffer = j.as_bytes().to_vec();
                            }
                        } else {
                            //Wrong sorting of query parameter
                            let error = JsonMessage {error: "First query parameter must be ID"};
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            let j = serde_json::to_string(&error).unwrap();
                            response.headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
                            buffer = j.as_bytes().to_vec();
                        }
                    },
                    None => {
                        buffer = format!("id missing").as_bytes().to_vec();
                        *response.status_mut() = StatusCode::NOT_FOUND;
                    }
                }
            },
            (&Method::GET, _) => {
                if !Server::load_data(&mut buffer, format!("{}{}", web_path, req.uri().path())).await {
                    *response.status_mut() = StatusCode::NOT_FOUND;
                }
            },
            (_,_) => {
                *response.status_mut() = StatusCode::NOT_FOUND;
                *response.body_mut() = Body::from("404 - Not found");
            }
        }
        *response.body_mut() = Body::from(buffer);

        Ok(response)
    }
    async fn load_data(buffer: &mut Vec<u8>, path: String) -> bool {
        let f = File::open(path.clone());
        match f.await {
            Ok(mut s) => {
                s.read_to_end(buffer).await.unwrap();
                true
            },
            Err(_) => {
                let notfound = format!("[.{}] File Not Found", path);
                *buffer = notfound.as_bytes().to_vec();    
                false
            }
        }
    }

    /*async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    }*/
}
