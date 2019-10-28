use actix_web::{
    web,
    App,
    HttpResponse,
    HttpServer,
};
use failure::Error;

use crate::util::config::Config;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    pub bind_address: String,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            bind_address: "0.0.0.0:8081".to_string(),
        }
    }
}

pub fn main(config: &Config) -> Result<(), Error> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/auth")
                    .route("", web::to(|| HttpResponse::Ok()))
                    .route("/login", web::to(|| HttpResponse::Ok()))
            )
    })
        .bind(&config.server.bind_address[..])
        .unwrap()
        .run()
        .unwrap();

    Ok(())
}