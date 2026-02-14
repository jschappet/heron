use crate::schema::offers;
use crate::schema::offers::dsl::*;
use crate::types::JsonField;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Identifiable, Serialize, Deserialize)]
pub struct Offer {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub offer: String,
    pub request: String,
    pub location: Option<String>,
    pub contact_link: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub details: JsonField,
}

// implement Default
impl Default for Offer {
    fn default() -> Self {
        Self {
            title: String::new(),
            offer: String::new(),
            request: String::new(),
            location: Some(String::new()),
            contact_link: Some(String::new()),
            start_date: chrono::Utc::now().naive_utc(),
            end_date: None,
            id: 0,
            user_id: 0,
            created_at: chrono::Utc::now().naive_utc(),
            details: JsonField(serde_json::json!({})),
        }
    }
}

#[derive(AsChangeset, Debug, Deserialize, Insertable, Default, Serialize)]
#[diesel(table_name = crate::schema::offers)]
pub struct OfferChangeset {
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub offer: Option<String>,
    pub request: Option<String>,
    pub location: Option<String>,
    pub contact_link: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub details: JsonField,
}

// --- CRUD Functions ---

pub fn create_offer(conn: &mut SqliteConnection, new_offer: OfferChangeset) -> QueryResult<Offer> {
    diesel::insert_into(offers::table)
        .values(new_offer)
        .execute(conn)?;
    offers.order(id.desc()).first(conn)
}

pub fn get_offer(conn: &mut SqliteConnection, offer_id_val: i32) -> QueryResult<Offer> {
    offers.find(offer_id_val).first(conn)
}

pub fn get_user_offers(conn: &mut SqliteConnection, uid: i32) -> QueryResult<Vec<Offer>> {
    offers.filter(user_id.eq(uid)).load(conn)
}

pub fn get_offers(conn: &mut SqliteConnection) -> QueryResult<Vec<Offer>> {
    offers.filter(user_id.is_not_null()).load(conn)
}

pub fn update_offer(
    conn: &mut SqliteConnection,
    offer_id_val: i32,
    changes: OfferChangeset,
) -> QueryResult<Offer> {
    diesel::update(offers.find(offer_id_val))
        .set(changes)
        .execute(conn)?;
    offers.find(offer_id_val).first(conn)
}

pub fn delete_offer(conn: &mut SqliteConnection, offer_id_val: i32) -> QueryResult<usize> {
    diesel::delete(offers.find(offer_id_val)).execute(conn)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_support::db::setup_test_db;

    
    #[test]
    fn create_offer_logic_works() {
    
        let (_tmp_dir, pool, this_user_id) 
            = setup_test_db();
        let mut conn = pool.get().unwrap();

        let new_offer = OfferChangeset {
            user_id: Some(this_user_id),
            title: Some("Test Offer".to_string()),
            offer: Some("offer text".to_string()),
            request: Some("request text".to_string()),

            ..Default::default()
        };

        let result = create_offer(&mut conn, new_offer).unwrap();
        let _created_id= result.id;
        log::info!("Offer created: {:?}", result);


        assert_eq!(result.title, "Test Offer");

        delete_offer(&mut conn, result.id).unwrap();
        use diesel::result::Error as DieselError;

        log::info!("Offer Deleted: ");
       match get_offer(&mut conn, result.id) {
        Ok(_) => panic!("Failed to Delete Offfer {}", result.id),
        Err(DieselError::NotFound) => {
        // ✅ expected
        },
        Err(e) => panic!("Unexpected DB error: {:?}", e) ,
       }
        
    }



    #[test]
    fn test_offer_update() {

        let (_tmp_dir, pool, this_user_id) 
            = setup_test_db();
        let mut conn = pool.get().unwrap();

        let new_offer = OfferChangeset {
            user_id: Some(this_user_id),
            title: Some("Test Offer".to_string()),
            offer: Some("offer text".to_string()),
            request: Some("request text".to_string()),

            ..Default::default()
        };

        let orig = create_offer(&mut conn, new_offer).unwrap();
        let new_offer = OfferChangeset {
            title:  Some("New Title".to_string()),
            user_id: Some(orig.user_id),
            offer: Some(orig.offer),
            request: Some(orig.request),
            location: orig.location,
            contact_link: orig.contact_link,
            start_date: Some(orig.start_date),
            end_date: orig.end_date,
            details: orig.details,
            

        } ;

        let new_offer = update_offer(&mut conn, orig.id, new_offer).unwrap();
        assert_eq!(new_offer.title,"New Title");

        
    }
}
