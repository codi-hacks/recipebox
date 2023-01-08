use actix_web::{
    HttpResponse,
    Responder,
    get,
    web::{self, Data}
};
use std::sync::Mutex;
use crate::data::DataStore;

#[get("/")]
async fn index(data: Data<Mutex<DataStore>>) -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(include_str!("../../cache/index.html"))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(index);
}
