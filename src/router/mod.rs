use actix_web::{
    HttpResponse,
    HttpRequest,
    Responder,
    get,
    web::{self, Data}
};
use std::{sync::Mutex};
use crate::data::DataStore;

#[get("/dashboard")]
async fn dashboard(req: HttpRequest) -> impl Responder {
    let data = req.app_data::<Data<Mutex<DataStore>>>().unwrap().lock().unwrap();
    HttpResponse::Ok().content_type("text/html").body(data.dashboard.clone())
}

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let data = req.app_data::<Data<Mutex<DataStore>>>().unwrap().lock().unwrap();
    HttpResponse::Ok().content_type("text/html").body(data.index.clone())
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(index);
}
