use std::{fs, io, path::Path, sync::Mutex};
use actix_web::web::Data;
use crate::args::Args;

pub struct DataStore {
    pub index: String,
    pub dashboard: String
}

impl DataStore {
    pub fn new(args: &Args) -> Result<Data<Mutex<DataStore>>, io::Error> {
        let index_path: String = Path::new(&args.cache).join("index.html").display().to_string();
        let dashboard_path: String = Path::new(&args.cache).join("dashboard.html").display().to_string();

        Ok(Data::new(Mutex::new(DataStore {
            index: fs::read_to_string(index_path)?,
            dashboard: fs::read_to_string(dashboard_path)?
        })))
    }
}
