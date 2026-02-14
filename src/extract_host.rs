// This file is no longer used directly, but the logic has been moved to middleware/host.rs for better integration with request handling.

fn extract_host(req: &HttpRequest) -> String {
    req.headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h)) // drop port
        .map(|h| h.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}



fn normalize_host(raw: &str) -> String {
    raw.split(',')
        .next()                // proxies may append
        .unwrap_or(raw)
        .trim()
        .split(':')            // strip port
        .next()
        .unwrap()
        .to_lowercase()
}


pub trait HostResolver {
    fn resolve_host_id(&self, host: &str) -> Result<i32, AppError>;
}


pub fn resolve_host_id(
    conn: &mut SqliteConnection,
    normalized_host: &str,
) -> Result<i32, AppError> {
    use crate::schema::hosts::dsl::*;

    if let Some(id) = hosts
        .filter(domain.eq(normalized_host))
        .select(id)
        .first::<i32>(conn)
        .optional()
        .map_err(AppError::Db)?
    {
        return Ok(id);
    }

    // Unknown host: insert or handle explicitly
    let new_host = NewHost {
        domain: normalized_host.to_string(),
        active_flag: false, // 👈 critical: not trusted yet
        first_seen_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(hosts)
        .values(&new_host)
        .execute(conn)
        .map_err(AppError::Db)?;

    // Re-fetch ID
    hosts
        .filter(domain.eq(normalized_host))
        .select(id)
        .first::<i32>(conn)
        .map_err(AppError::Db)
}
