use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use configparser::ini::Ini;

#[derive(Clone)]
pub struct Config {
    //ServerLogin
    pub serverlogin_url: String,
    pub serverlogin_username: String,
    pub serverlogin_password: String,
    //Scheduler
    pub scheduler_duration: u64,
    //Server
    pub server_ip: IpAddr,
    pub server_port: u16,
    pub server_web_path: String,
    //Database
    pub database_customer_data: String
}


impl Config {
    pub fn new() -> Self {
        let mut config = Ini::new();
        config.load("kraken.ini").expect("Konnte kraken.ini nicht laden.");
        Self { 
            //ServerLogin
            serverlogin_url: config.get("requester", "url")
                .expect("[requester] 'url' konnte nicht geparst werden."),
            serverlogin_username: config.get("requester", "user")
                .expect("[requester] 'user' konnte nicht geparst werden."), 
            serverlogin_password: config.get("requester", "password")
                .expect("[requester] 'password' konnte nicht geparst werden."),
            //Scheduler
            scheduler_duration: config.getuint("scheduler", "update-rate")
                .expect("[scheduler] 'update-rate' konnte nicht geparst werden.")
                .expect("[scheduler] 'update-rate' konnte wert nicht zu u64 parsen."),
            //Server
            server_ip: IpAddr::V4(
                Ipv4Addr::from_str(
                        config.get("server", "ip")
                            .expect("[server] 'ip' konnte nicht geparst werden.")
                            .as_str()
                    )
                    .expect("[server] Konnte ip zu Ipv4Addr konvertieren")
            ),
            server_port: config.getuint("server", "port")
                .expect("[server] 'port' konnte nicht geparst werden.")
                .expect("[server] 'port' Konnte ip zu u16 konvertiert werden.") as u16,
            server_web_path: config.get("server", "web-path")
                .expect("[server] 'web-path' konnte nicht geparst werden."),
            database_customer_data: config.get("database", "customer-data")
                .expect("[database] 'customer-data' konnte nicht geparst werden."),
        }
    }
}

