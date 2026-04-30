use crate::{errors::app_error::AppError, routes::{register, RoutePath}, services::hosts::HostDomain, types::method::Method, validator::AuthContext};
use actix_web::{HttpResponse,  Scope, web};



//#[get("")]
pub async fn list_all_hosts(
    host_domain: web::Data<HostDomain>,
    admin: AuthContext
) -> Result<HttpResponse, AppError> {

    
    let result = host_domain.get_host_list()?;
    

    Ok(HttpResponse::Ok().json(result))
}


//.service(register(Method::POST, "/details", update_user_details_api, true))


// pub fn scope() -> Scope {
//     let root_path= "/hosts";
//     web::scope("")
//         .service(register(root_path, Method::GET, "", list_all_hosts, true))
        
// }

pub fn scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("host_list", Method::GET, path.as_str(), "", list_all_hosts, crate::types::MemberRole::Public))
}



pub fn admin_scope(path: &RoutePath) -> Scope {
    web::scope("")
        .service(register("admin_host_list", Method::GET, path.as_str(), "list", list_all_hosts, crate::types::MemberRole::Member))
}


