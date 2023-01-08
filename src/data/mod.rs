use std::sync::Mutex;
use actix_web::web::Data;

pub struct DataStore {
    foo: usize
}

impl DataStore {
    pub fn new(foo: usize) -> Data<Mutex<DataStore>> {
        Data::new(Mutex::new(DataStore {
            foo: foo
        }))
    }
}
