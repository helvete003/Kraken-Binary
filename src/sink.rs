use std::sync::Arc;
use std::collections::HashMap;
use futures::lock::Mutex;
use chrono::{DateTime, Utc, Local};
use serde::{Deserialize, Serialize};

//pub type DataSink = Arc<Mutex<Vec<Sink>>>;
pub type DataSink = Arc<Mutex<Vec<ResponseData>>>;


#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ResponseData {
    pub statuscode: u16,
    pub id: u32,
    pub customer: String,
    pub url: String,
    pub error: String,
    pub errorcode: u16,
    pub lastupdate: String,
    pub body: KrakenResponse
}

impl ResponseData {
    pub fn new() -> Self {
        let now: DateTime<Utc> = Utc::now();
        let localtime: DateTime<Local> = DateTime::from(now);
        Self {
            lastupdate: localtime.format("%d.%m.%Y %H:%M:%S").to_string(),
            ..Default::default()
        }
    }
}

/**
*   Die struktur für die daten die vom Kraken script vom Server kommen
*/
#[derive(Serialize, Deserialize,  Default, Debug)]
#[serde(default)]
pub struct KrakenResponse {
    platform:Platform,
    interpreter:Interpreter,
    web_server:WebServer,
    database_server:DatabaseServer,
    os:Os,
    kraken:KrakenVersion,
    extra: HashMap<String,Vec<Extra>>
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Platform {
    name: String,
    version: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Interpreter {
    name: String,
    version: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct WebServer {
    name: String,
    server_signature: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DatabaseServer {
    client_info: String,
    server_info: String,
    host_info: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Os {
    name: String,
    raw: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KrakenVersion {
    version: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Extra {
    name: String,
    version: String,
    url: String,
    alerts: String,
    active: bool
}