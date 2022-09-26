use tokio::sync::mpsc::{Sender, Receiver};

#[derive(Debug)]
pub enum Command {
    Refresh,
    Scheduler(&'static str),
    Requester(&'static str),
    Server(&'static str)
}


pub struct Manager {
    request_locked: bool
}

impl Manager {
    pub fn new() -> Self {
        Self {
            request_locked: true
        }
    }
    pub async fn run(&mut self, mut rx: Receiver<Command>, requester_tx: Sender<Command>, _server_tx: Sender<Command>) {
        //First start, send Setup to requester
        requester_tx.send(Command::Refresh).await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Server(s_cmd) => match s_cmd {
                    //"refresh" => requester_tx.send(Command::Refresh).await.unwrap(),
                    "refresh" => {
                        self.send_request(&requester_tx, Command::Refresh).await;
                        self.request_locked = true;
                    },
                    _ => {}
                },
                Command::Scheduler(s_cmd) => match s_cmd {
                    "refresh" => {
                        self.send_request(&requester_tx, Command::Refresh).await;
                        self.request_locked = true;
                    },
                    _ => {}
                },
                Command::Requester(s_cmd) => match s_cmd {
                    //Setup ist fertig, wir entsperren die request wieder
                    "done" => self.request_locked = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }
    async fn send_request(&self, tx: &Sender<Command>, cmd: Command) {
        //println!("{:#?}   {}", cmd, self.request_locked);
        if !self.request_locked {
            tx.send(cmd).await.unwrap();
        }
    }
}