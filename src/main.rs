use std::io::Error;

use actix_web::{
    App,
    HttpServer,
    middleware::Logger
};
use clap::Parser;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config as LogConfig, TermLogger, TerminalMode};

use recipebox::args::Args;
use recipebox::data::DataStore;
use recipebox::router::init_routes;
//use recipebox::forms::recipe::Recipe;
use recipebox::setup::check_for_directories;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    TermLogger::init(LevelFilter::Debug, LogConfig::default(), TerminalMode::Stdout, ColorChoice::Auto).unwrap();

    let args = Args::parse();
    let address = format!("{}:{}", args.host, args.port);

    check_for_directories();


    let data_store = DataStore::new(33);

    let server = HttpServer::new(move ||
        App::new()
            .app_data(data_store.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .configure(init_routes)
    );

    info!("running on {}:{}", args.host, args.port);
    server.bind(address)?.run().await




    /*
    router.on("/add-recipe", |_, request| {
        match Recipe::from_request(&request.body) {
            Ok(recipe) => {
                recipe.markdown_to_html();
                let message = b"You found the secret message!";
                SendResponse(Response {
                    status: status::OK,
                    headers: header_map![(header::CONTENT_LENGTH, "29")],
                    body: message.to_vec(),
                })
            },
            Err(message) => {
                SendResponse(Response {
                    status: status::BAD_REQUEST,
                    headers: header_map![(header::CONTENT_LENGTH, message.len().to_string())],
                    body: message.to_vec()
                })
            }
        }
    });

    router.route("/dashboard", file_router("./pages/dashboard.hbs"));
    router.route("/", file_router("./pages/index.hbs"));
    */
}

/*
fn file_router(directory: &'static str) -> Router {
    let mut router = Router::new();

    let cache: RwLock<HashMap<String, Arc<Response>>> = RwLock::new(HashMap::new());

    router.on_prefix("", move |uri, _| {
        let mut path = String::from(directory);
        path.push_str(uri);

        if path.ends_with('/') {
            path.push_str("index.hbs");
        }
        if path.ends_with(".hbs") {
            path = path
                .replace("./pages", "cache")
                .replace(".hbs", ".html");
        }

        if let Some(response) = cache.read().unwrap().get(&path) { // read lock gets dropped after if statement
            return SendResponseArc(Arc::clone(response));
        }

        let response = Arc::new(file_response(&path));

        cache.write().unwrap().insert(path, Arc::clone(&response));

        SendResponseArc(response)
    });

    router
}

fn file_response(file_path: &str) -> Response {
    if let Ok(contents) = fs::read(file_path) {
        let headers = header_map![
            (header::CONTENT_LENGTH, contents.len().to_string()),
            (header::CONTENT_TYPE, get_content_type(file_path))
        ];

        return Response { status: status::OK, headers, body: contents };
    }
    status::NOT_FOUND.into()
}
*/
