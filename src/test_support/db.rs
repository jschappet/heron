use tempfile::NamedTempFile;
use crate::db::DbPool;

#[allow(dead_code)]
pub fn setup_test_db() -> (NamedTempFile, DbPool, i32) {
    use crate::models::users::create_user;
    use diesel::SqliteConnection;
    use diesel::r2d2::{ConnectionManager, Pool};

    let tmp = NamedTempFile::new().unwrap();
    let db_url = tmp.path().to_str().unwrap().to_string();

    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().build(manager).unwrap();

    {
        let mut conn = pool.get().unwrap();
        let _ = crate::db::run_migrations(&mut conn);

        let this_user_id = create_user(&mut conn,
             "testuser", Some("test@example.com"))
            .unwrap().id;
        return (tmp, pool, this_user_id);
    }
}
