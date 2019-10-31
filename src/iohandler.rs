use std::{fs::File, io::BufReader, io::Read};
use std::sync::mpsc::{
    channel,
    Sender,
    Receiver,
};

use failure::{
    format_err,
    Error,
};

use crate::{
    loaders::{
        meshes_from_erm,
        mesh_from_bmp,
    },
    display::Mesh,
};

pub enum Request {
    LoadFile(String),
    LoadModel(String),
    LoadTerrain(LoadTerrainRequest),
}

pub enum Response {
    FileLoaded(FileLoaded),
    ModelLoaded(ModelLoaded),
    TerrainLoaded(TerrainLoaded),
    Error,
}

pub struct LoadTerrainRequest {
    pub path: String,
    pub scale: f32,
}

pub struct FileLoaded {
    pub path: String,
    pub buf: Vec<u8>,
}

pub struct ModelLoaded {
    pub path: String,
    pub meshes: Vec<Mesh>,
}

pub struct TerrainLoaded {
    pub path: String,
    pub mesh: Mesh,
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
                        Request::LoadFile(path) => {
                            if let Ok(buf) = self.load_file(&path[..]) {
                                Response::FileLoaded(
                                    FileLoaded { path, buf }
                                )
                            } else {
                                Response::Error
                            }
                        },
                        Request::LoadModel(path) => {
                            if let Ok(meshes) = self.load_model(&path[..]) {
                                Response::ModelLoaded(
                                    ModelLoaded { path, meshes }
                                )
                            } else {
                                Response::Error
                            }
                        },
                        Request::LoadTerrain(LoadTerrainRequest { path, scale }) => {
                            if let Ok(mesh) = self.load_terrain(&path[..], scale) {
                                Response::TerrainLoaded(
                                    TerrainLoaded { path, mesh }
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
        meshes_from_erm(path)
    }

    fn load_terrain(&self, path: &str, scale: f32) -> Result<Mesh, Error> {
        mesh_from_bmp(path, scale)
    }

    fn load_file(&self, path: &str ) -> Result<Vec<u8>, Error>
    {
        let mut image_reader = BufReader::new(
            File::open(path)
                .map_err(|e| {
                    format_err!("Unable to open {}: {:?}", path, e)
                })?
        );

        let mut buf = Vec::new();
        image_reader.read_to_end(&mut buf)?;

        Ok(buf)
    }
}