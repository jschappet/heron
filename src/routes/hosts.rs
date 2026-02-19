use crate::{errors::app_error::AppError, routes::register, services::hosts::HostDomain, types::method::Method, validator::AuthContext};
use actix_web::{HttpResponse,  Scope, post, get, web};
use serde::{Deserialize, Serialize};



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

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path= parent_path.join("/");

    web::scope("")
        .service(register("host_list", Method::GET, &full_path ,"",list_all_hosts,crate::types::MemberRole::Public,))
}



pub fn admin_scope(parent_path: Vec<&str>) -> Scope {
    let full_path= parent_path.join("/");
    web::scope("")
        .service(register("admin_host_list", Method::GET, &full_path, "list", list_all_hosts, crate::types::MemberRole::Member))
}


