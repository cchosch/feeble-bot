use crate::db::gen_id;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    password: String,
    pub email: String,
    created_at: NaiveDateTime,
    pub verified_email: bool,

    #[serde(skip_serializing)]
    pub banned: bool,
    #[serde(skip_serializing)]
    pub kicked_until: NaiveDateTime,
    pub flags: i64,
}

// UserFlags
pub const OWNER: i64 = 1 << 0;
pub const STAFF: i64 = 1 << 1;

const BCRYPT_COST: u32 = optimal_cost();

/// Binary search for optimal cost depending on hardware. Shoots for 250ms hashing delay
const fn optimal_cost() -> u32 {
    return 8;
}

impl User {
    pub fn new(username: String, password: String, email: String) -> User {
        let hashed_password = bcrypt::hash(Sha256::digest(password), BCRYPT_COST).unwrap();
        User {
            id: gen_id(),
            username,
            password: hashed_password,
            email,
            flags: 0,
            created_at: Utc::now().naive_utc(),
            verified_email: false,
            banned: false,
            kicked_until: NaiveDateTime::UNIX_EPOCH,
        }
    }

    pub fn compare_passwords(&self, pass: String) -> bool {
        bcrypt::verify(Sha256::digest(pass), self.password.clone().as_str()).unwrap_or(false)
    }

    pub fn created_date(&self) -> DateTime<Utc> {
        return self.created_at.and_utc().clone();
    }


    pub fn set_flag(&mut self, flag: i64, new_value: bool) -> &mut Self {
        if new_value {
            self.flags = self.flags | flag;
        } else {
            self.flags = self.flags ^ flag;
        }

        return self;
    }
}