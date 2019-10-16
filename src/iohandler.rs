use std::sync::mpsc::{
    channel,
    Sender,
    Receiver,
};

use failure::Error;

use crate::{
    loaders::meshes_from_erm,
    renderer::Mesh,
};

pub enum Request {
    LoadModel(String),
}

pub enum Response {
    ModelLoaded(ModelLoaded),
    Error,
}

pub struct ModelLoaded {
    pub path: String,
    pub meshes: Vec<Mesh>,
}

pub struct IOHandler {
    req_rx: Receiver<Request>,
    res_tx: Sender<Response>,
}

impl IOHandler {
    pub fn new() -> (IOHandler, (Sender<Request>, Receiver<Response>)) {
        let (req_tx, req_rx) = channel::<Request>();
        let (res_tx, res_rx) = channel::<Response>();

        let handler = IOHandler {
            req_rx,
            res_tx,
        };

        (handler, (req_tx, res_rx))
    }

    pub fn run(self) {
        loop {
            let response = match self.req_rx.recv() {
                Ok(request) => {
                    match request {
                        Request::LoadModel(path) => {
                            if let Ok(meshes) = self.load_model(&path[..]) {
                                Response::ModelLoaded(
                                    ModelLoaded { path, meshes }
                                )
                            } else {
                                Response::Error
                            }
                        },
                    }
                },
                Err(_) => break,
            };

            self.res_tx.send(response)
                .unwrap_or_else(|err| {
                    log::error!("failed to send io response: {}", err);
                });
        }
    }

    fn load_model(&self, path: &str) -> Result<Vec<Mesh>, Error> {
        meshes_from_erm(&path)
    }
}