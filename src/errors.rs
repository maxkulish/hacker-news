use actix_web::HttpResponse;
use std::fmt;

#[derive(Debug)]
pub enum ServerError {
    ArgonauticError,
    DieselError,
    EnvironmentError,
    R2D2Error,
    UserError(String)
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Test")
    }
}

impl actix_web::error::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::ArgonauticError => HttpResponse::InternalServerError().json("Argonautica Error"),
            ServerError::DieselError => HttpResponse::InternalServerError().json("Diesel Error"),
            ServerError::EnvironmentError => HttpResponse::InternalServerError().json("Environment Error"),
            ServerError::R2D2Error => HttpResponse::InternalServerError().json("r2d2 Error"),
            ServerError::UserError(data) => HttpResponse::InternalServerError().json(data),
        }
    }
}

impl From<std::env::VarError> for ServerError {
    fn from(_: std::env::VarError) -> ServerError {
        ServerError::EnvironmentError
    }
}

impl From<r2d2::Error> for ServerError {
    fn from(_: r2d2::Error) -> ServerError {
        ServerError::R2D2Error
    }
}

impl From<diesel::result::Error> for ServerError {
    fn from(err: diesel::result::Error) -> ServerError {
        match err {
            diesel::result::Error::NotFound => {
                log::error!("db error: {:?}", err);
                ServerError::UserError("Username not found.".to_string())
            },
            _ => ServerError::DieselError
        }
    }
}

impl From<argonautica::Error> for ServerError {
    fn from(_: argonautica::Error) -> ServerError {
        ServerError::ArgonauticError
    }
}