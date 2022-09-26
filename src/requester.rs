use futures::future::join_all;
use tokio::sync::mpsc;
use reqwest::header;

use crate::sink::{KrakenResponse, ResponseData, DataSink};

use crate::manager;
use crate::config::Config;
use crate::database::*;


pub struct Requester {
    config: Config,
    memory: DataSink,
}

impl Requester {
    pub fn new(config: Config, memory: DataSink) -> Self {
        Self {
            config,
            memory
        }
    }
    pub async fn run(&self, tx: mpsc::Sender<manager::Command>, mut rx: mpsc::Receiver<manager::Command>) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                manager::Command::Refresh => {
                    self.refresh().await;
                    tx.send(manager::Command::Requester("done")).await.unwrap();
                },
                _ => {}
            }
        }
    }
    /** 
    *   Refresh holt die liste neu und macht die Requests an den Server
    */
    async fn refresh(&self) {
        let mut responses:Vec<ResponseData> = Vec::new();
        match Database::new(&self.config.database_customer_data) {
            Ok(mut db) => {
                let mut request_list = Vec::new();
                match db.get_customer_data() {
                    Ok(data) => {
                        for e in data.iter() {
                            if e.kunde != None && e.url != None && e.password != None {
                                /*let Some(url) = &e.url;
                                let Some(password) = &e.password;*/
                                let domain = format!("{}?key={}", e.url.as_ref().unwrap(), e.password.as_ref().unwrap());
                                let kunde = e.kunde.as_ref().unwrap();
                                request_list.push(self.request(domain.to_string(),kunde.to_string(), e.id.unwrap()));
                            }
                        }
                    },
                    Err(error) => {
                        let mut error_data = ResponseData::new();
                        error_data.statuscode = 901;
                        error_data.errorcode = 4;
                        error_data.error = format!("Mysql Error. {}", error);
                        responses.push(error_data);
                    }
                }
                responses = join_all(request_list).await;
            },
            Err(error) => {
                let mut error_data = ResponseData::new();
                error_data.statuscode = 901;
                error_data.errorcode = 4;
                error_data.error = format!("Mysql Login Error. Bitte Prüfen! {}", error);
                responses.push(error_data);
            }
        }
        /*match self.get_customer_data().await {
            Ok(customer_data) => {
                let mut request_list = Vec::new();
                if let serde_json::Value::Array(obj) = customer_data {
                    for e in obj.iter() {
                        let domain = if let serde_json::Value::String(domain) = e.get("url").unwrap() {
                            domain
                        } else {
                            panic!("Fehler beim laden der URL");
                        };
                        let kunde = if let serde_json::Value::String(kunde) = e.get("kunde").unwrap() {
                            kunde
                        } else {
                            panic!("Fehler beim laden des Kundennamen");
                        };
                        request_list.push(self.request(domain.to_string(),kunde.to_string()));
                    }
                }
                responses = join_all(request_list).await;
            }
            Err(error) => {
                let mut error_data = ResponseData::new();
                error_data.statuscode = 901;
                error_data.errorcode = 4;
                error_data.error = format!("Logindaten oder Url für den Requester ist falsch. Bitte Prüfen! {}", error);
                responses.push(error_data);
            }
        }*/
        let mut memory = self.memory.lock().await;
        *memory = responses;
    }
    /*
    Get a json with all the URLs of the customers
    */
    #[allow(dead_code)]
    async fn get_customer_data(&self) -> Result<serde_json::Value, reqwest::Error> {
        let user_auth = format!("Basic {}", 
            base64::encode(
                format!("{}:{}", self.config.serverlogin_username,self.config.serverlogin_password)
            )
        );
        //println!("{:?}", user_auth);
        let mut headers = header::HeaderMap::new();
        let mut auth_value = header::HeaderValue::from_str(user_auth.as_str()).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let response =  client.get(self.config.serverlogin_url.as_str()).send().await?;
        response.json().await
    }

    /*
    This method is doing the requests to the server
    */
    async fn request(&self, url: String, customer: String, id: u32) -> ResponseData {
        let mut data = ResponseData::new();
        data.customer = customer.clone();
        data.id = id;
        match reqwest::get(&url).await {
            Ok(response) => {
                let headers = response.headers();

                data.statuscode = response.status().as_u16();
                data.url = response.url().to_string();
                if headers.contains_key("Kraken") && headers.contains_key(reqwest::header::CONTENT_TYPE) {
                    let content_type = headers.get(reqwest::header::CONTENT_TYPE).unwrap(); 
                    if content_type == "application/json" {
                        match response.text().await {
                            Ok(body) => {
                                let no_bom = Requester::string_to_no_bom(&body); //Nötige funktion wegen dem BOM 
                                match serde_json::from_str::<KrakenResponse>(no_bom) {
                                    Ok(json) => data.body = json,
                                    Err(error) => {
                                        data.errorcode = 3;
                                        data.error = format!("Fehler beim entpacken des Jsons: \n {}", error);
                                    }
                                };
                            },
                            Err(error) => {
                                data.errorcode = 3;
                                data.error = format!("Fehler beim entpacken des Body: \n {}", error);
                            }
                        };
                    } else {
                        data.errorcode = 1;
                        data.error = "Server Antwortet ohne application/json".to_string();
                    }
                } else {
                    data.errorcode = 2;
                    data.error = "Kraken System nicht gefunden!".to_string();
                }
                data
            }
            Err(error) => {
                data.statuscode = 902;
                data.url = url.clone();
                data.errorcode = 3;
                data.error = error.to_string();
                data
            }
        }
    }
    /*
    *   Diese methode ist nur dafür da um einen Möglichen BOM aus dem body entfernen
    */
    fn string_to_no_bom(body_text: &String) -> &str {
        let body_bytes = body_text.as_bytes();
        if body_bytes[0] == 0xef && body_bytes[1] == 0xbb && body_bytes[2] == 0xbf {
            return &body_text[3..];
        }
        &body_text[..]
    }
}
