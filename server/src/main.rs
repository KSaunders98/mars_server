use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use common::models::Tile;

pub fn establish_connection() -> Result<MysqlConnection, ConnectionError> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
}

fn err_not_found() -> Result<Response<Body>, hyper::Error> {
    let mut err = Response::default();
    *err.status_mut() = StatusCode::NOT_FOUND;
    Ok(err)
}

fn err_internal() -> Result<Response<Body>, hyper::Error> {
    let mut err = Response::default();
    *err.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    Ok(err)
}

async fn handle_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let params: HashMap<String, String> = req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .collect()
    })
    .unwrap_or_else(HashMap::new);

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/elevation") => elev_req(&params),
        (&Method::GET, "/imagery") => imagery_req(&params),
        _ => err_not_found(),
    }
}

fn elev_req(params: &HashMap<String, String>) -> Result<Response<Body>, hyper::Error> {
    use common::schema::tiles;

    match params.get("id") {
        Some(id) => {
            match id.parse::<u32>() {
                Ok(id) => {
                    let connection = match establish_connection() {
                        Ok(c) => c,
                        Err(_) => return err_internal(),
                    };

                    let result = tiles::table.filter(tiles::id.eq(id))
                        .load::<Tile>(&connection);
                    
                    match result {
                        Ok(tiles) => {
                            if !tiles.is_empty() {
                                Ok(Response::new(Body::from(tiles[0].elevation_data.clone())))
                            } else {
                                err_not_found()
                            }
                        },
                        Err(_) => err_not_found(),
                    }
                },
                Err(_) => err_not_found(),
            }
        },
        None => err_not_found(),
    }
}

fn imagery_req(params: &HashMap<String, String>) -> Result<Response<Body>, hyper::Error> {
    use common::schema::tiles;

    match params.get("id") {
        Some(id) => {
            match id.parse::<u32>() {
                Ok(id) => {
                    let connection = match establish_connection() {
                        Ok(c) => c,
                        Err(_) => return err_internal(),
                    };

                    let result = tiles::table.filter(tiles::id.eq(id))
                        .load::<Tile>(&connection);
                    
                    match result {
                        Ok(tiles) => {
                            if !tiles.is_empty() {
                                Ok(Response::new(Body::from(tiles[0].imagery_data.clone())))
                            } else {
                                err_not_found()
                            }
                        },
                        Err(_) => err_not_found(),
                    }
                },
                Err(_) => err_not_found(),
            }
        },
        None => err_not_found(),
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_req))
    });

    let server = Server::bind(&addr).serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());

    server.await?;

    Ok(())
}
