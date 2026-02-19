use actix_web::Route;

#[derive(Copy, Clone, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

impl Method {
    pub fn to_route(self) -> Route {
        match self {
            Method::GET => actix_web::web::get(),
            Method::POST => actix_web::web::post(),
            Method::PUT => actix_web::web::put(),
            Method::DELETE => actix_web::web::delete(),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
        }
    }
}