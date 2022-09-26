mod config;
mod runtime;
mod server;
mod requester;
mod scheduler;
mod manager;
mod sink;
mod database;

use std::error::Error;
use std::sync::Arc;
use futures::lock::Mutex;

use sink::{DataSink, ResponseData};
use runtime::*;
use scheduler::*;
use crate::server::Server;
use crate::requester::Requester;
use crate::config::Config;

fn setup_response() -> ResponseData {
    let mut init_memory = ResponseData::new();
    init_memory.statuscode = 900;
    init_memory.errorcode = 5;
    init_memory.error = "System im Setup... bitte aktualisieren Sie die Webseite".to_string();
    init_memory
}

fn main() -> Result<(), Box<dyn Error>> {
    //Load kraken.ini config file
    let config = Config::new();

    //We create a ResponseData with some information. Because the first boot can lead to an empty site on the frontend
    let init_memory = setup_response();
    //data_memory is the shared memory for the threads
    let data_memory: DataSink = Arc::new(Mutex::new(vec![init_memory]));
    println!("Kraken System");

    //Create Requester object
    let requester = Requester::new(
        config.clone(), 
        data_memory.clone()
    );
    //Create Scheduler object
    let scheduler = Scheduler::new(
        config.scheduler_duration
    );
    //Create Server object
    let server = Server::new(
        config.server_ip,
        config.server_port,
        config.server_web_path,
        data_memory.clone()
    );
    //Start the runtime with all the threads
    start_runtime(scheduler, requester, server);
    Ok(())
}
