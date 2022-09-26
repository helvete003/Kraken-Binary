use tokio::sync::mpsc;
use futures::future::join4;

use crate::scheduler::Scheduler;
use crate::server::Server;
use crate::requester::Requester;
use crate::manager::*;

#[tokio::main]
pub async fn start_runtime(scheduler: Scheduler, requester: Requester, server: Server) {
    let mut manager = Manager::new();
    
    let (request_tx, rx): (mpsc::Sender<Command>, mpsc::Receiver<Command>)  = mpsc::channel(64);
    let server_tx = request_tx.clone();
    let scheduler_tx = request_tx.clone();

    let (request_oneshot_tx, request_oneshot_rx): (mpsc::Sender<Command>, mpsc::Receiver<Command>) = mpsc::channel(64);
    let (server_oneshot_tx, mut _server_oneshot_rx): (mpsc::Sender<Command>, mpsc::Receiver<Command>)  = mpsc::channel(64);

    let tmanager = tokio::spawn(async move {
        manager.run(rx, request_oneshot_tx, server_oneshot_tx).await;
    });

    let tscheduler = tokio::spawn(async move {
        scheduler.run(scheduler_tx).await;
    });

    let trequester = tokio::spawn(async move {
        requester.run(request_tx, request_oneshot_rx).await;
    });

    let tserver = tokio::spawn(async move {
        server.run(server_tx).await;
    });

    let (t1,t2,t3,t4) = join4(tmanager, tscheduler, trequester,tserver).await;
    t1.unwrap();
    t2.unwrap();
    t3.unwrap();
    t4.unwrap();
    
    
    //ALte verison nutzen ein loop was zu 100% cpu leistung geführt hat
    /*loop {

    }*/
}
