use std::{net::ToSocketAddrs, fs::File, path::PathBuf};

use tiny_http::{Server, ResponseBox, Request, Response, Method};

use crate::indexer::Indexer;


fn unsupported_method(method: &Method) -> ResponseBox {
    Response::from_string(format!("Unsopported Method {method}")).boxed()
}

fn error404(url: &str) -> ResponseBox {
    Response::from_string(format!("Error 404 {url} not found"))
    .with_status_code(404)
    .boxed()
}

fn error500() -> ResponseBox {
    Response::from_string("Error 500 internal error")
    .with_status_code(500)
    .boxed()
}

fn send_file(url: &str) -> ResponseBox {

    let mut file_path = PathBuf::new();
    file_path.push("frontend");
    file_path.push(&url[1..]);
    if let Ok(file) = File::open(file_path) {
        Response::from_file(file).boxed()
    }
    else {
        error404(url)
    }

}

fn handle_request(mut request: Request, indexer: &Indexer) {
    let response = match (request.method(), request.url()) {
        (Method::Get, "/") => get_main_page(),
        (Method::Get, url) => send_file(url),

        (Method::Post, "/api/search") => {
            let mut buf = String::new();
            if let Err(_) = request.as_reader().read_to_string(&mut buf) {
                error500()
            }
            else {
                get_search_query_results(buf, indexer)
            }
        }
        (Method::Post, url) => send_file(url),

        (method, _) => unsupported_method(method),
    };

    request.respond(response).unwrap();
}

pub fn start_server(addr: impl ToSocketAddrs, indexer: &Indexer) -> Result<(), ()> {

    let server = Server::http(addr).map_err(|err| {
        eprintln!("ERROR cannot launch the server, {err}");
    })?;

    println!("server started on {} and ready", server.server_addr());
    server.incoming_requests().for_each(|request| handle_request(request, indexer));

    Ok(())
}

fn get_main_page() -> ResponseBox {
    send_file("/index.html")
}

fn get_search_query_results(query: String, indexer: &Indexer) -> ResponseBox {

    let results = indexer.get_docs_for_query(&query);
    let results = results.iter().map(|(p, _)| p).collect::<Vec<_>>();

    let results_str = serde_json::to_string(&results).unwrap();

    Response::from_string(&results_str)
    .boxed()
}