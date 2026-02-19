use crate::validator::AuthContext;
use crate::types::MemberRole;
use actix_web::{
    Error, FromRequest, HttpMessage, HttpResponse,
    body::{EitherBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;
use actix_web::dev::Payload;


/// Marker type
pub struct AdminMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AdminMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // 1️⃣ Check if AuthContext is already in extensions
            let auth = if let Some(auth) = req.extensions().get::<AuthContext>() {
                log::debug!("Got Auth");
                auth.clone()
            } else {
                // 2️⃣ Extract manually from request if missing
                
                log::debug!("Extracting Auth Manually");
                let mut payload = Payload::None;  // 👈 empty payload
                let auth = match AuthContext::from_request(req.request(), &mut payload).await {

                    Ok(a) => a,
                    Err(_) => {
                        return Ok(req.into_response(
                            HttpResponse::Unauthorized()
                                .finish()
                                .map_into_left_body::<B>(),
                        ));
                    }
                };
                // Store for downstream handlers
                req.extensions_mut().insert(auth.clone());
                auth
            };

           
            if !auth.is_admin() {
                return Ok(
                    req.into_response(HttpResponse::Forbidden().finish().map_into_left_body::<B>())
                );
            }

            // 4️⃣ Forward to next service and map its body to the right side
            match svc.call(req).await {
                Ok(res) => Ok(res.map_into_right_body::<BoxBody>()),
                Err(e) => Err(e),
            }
        })
    }
}
