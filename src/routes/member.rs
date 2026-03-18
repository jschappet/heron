use crate::domains::member_domain::MemberDomain;
use crate::middleware::host::HostContext;
use crate::{routes::register, types::method::Method, validator::AuthContext};
use actix_web::{HttpResponse, Responder, Scope, web};

pub async fn member_content(
    domain: web::Data<MemberDomain>, 
    auth: AuthContext, 
    host: HostContext
) -> impl Responder {
    //let mut conn = data.db_conn()?;
    //let _ = delete_membership(&mut conn, id.into_inner())?;
    match domain.member_content(host.0.id, auth.user_id) {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn member_flows(domain: web::Data<MemberDomain>, 
    auth: AuthContext, 
    host: HostContext
) -> impl Responder {
    //let mut conn = data.db_conn()?;
    //let _ = delete_membership(&mut conn, id.into_inner())?;
    match domain.member_flows(host.0.id, auth.user_id) {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn scope(parent_path: Vec<&str>) -> Scope {
    let full_path = parent_path.join("/");
    web::scope("")
        .service(register(
            "member_content",
            Method::GET,
            &full_path,
            "content",
            member_content,
            crate::types::MemberRole::Member,
        ))
        .service(register(
            "member_flows",
            Method::GET,
            &full_path,
            "flows",
            member_flows,
            crate::types::MemberRole::Member,
        ))
}
