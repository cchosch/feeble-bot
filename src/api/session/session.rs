use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use diesel::prelude::*;
use lazy_static::lazy_static;
use rand::{random};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use crate::schemas::User;
use crate::util::strip_quotes;
use crate::schema::sessions;


lazy_static! {
    pub static ref SESS_TIMEOUT: Duration = Duration::try_days(5).unwrap();
}

pub static SESSION_COOKIE_NAME: &'static str = "sid";

#[derive(Serialize, Deserialize, Clone, Debug, Selectable, Queryable, Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DBSession {
    sess_id: String,
    sess_cookie: String,
    expiry: NaiveDateTime,
    uid: Option<String>,
    data: Option<Value>,
}

impl Default for DBSession {
    fn default() -> Self {
        let id = hex::encode(random::<[u8; 32]>());
        DBSession {
            sess_id: id.clone(),
            sess_cookie: Session::generate_cookie(id),
            expiry: Utc::now().add(SESS_TIMEOUT.clone()).naive_utc(),
            uid: None,
            data: Some(Value::Object(serde_json::Map::new())),
        }
    }
}

impl DBSession {
    pub fn to_session(&self) -> Session {
        let id = self.clone().sess_id;
        Session {
            id: id.clone(),
            expiry: self.expiry.and_utc(),
            data: self.data_to_hashmap().unwrap_or(HashMap::new()),
            cookie: Session::generate_cookie(id),
            user_id: self.uid.clone(),
            ..Default::default()
        }
    }

    pub fn data_to_hashmap(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut out: HashMap<String, String> = HashMap::new();

        match self.data.as_ref() {
            Some(s) => match s {
                Value::Object(_o) => {
                    return Ok(serde_json::from_value(s.to_owned())?);
                }
                _ => {
                    let data_val = serde_json::to_string(s).unwrap_or(String::new());
                    out.insert(String::from("data"), strip_quotes(data_val));
                }
            },
            None => {}
        }
        Ok(out)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub expiry: DateTime<Utc>,
    data: HashMap<String, String>,
    has_changed: bool,
    user_id: Option<String>,
    pub cookie: String,
    to_delete: bool,
}

impl Default for Session {
    fn default() -> Self {
        let id = hex::encode(random::<[u8; 32]>());
        Session {
            id: id.clone(),
            expiry: Utc::now().add(SESS_TIMEOUT.clone()),
            data: HashMap::new(),
            has_changed: false,
            user_id: None,
            cookie: Session::generate_cookie(id),
            to_delete: false,
        }
    }
}

impl Session {
    pub fn new() -> Self {
        return Session::default();
    }

    pub fn expired_and_none(self) -> Option<Self> {
        if self.is_expired() {
            return None;
        }
        return Some(self.clone());
    }

    pub fn destroy(&mut self) -> &mut Self {
        self.to_delete = true;
        self
    }

    pub fn to_del(&self) -> bool {
        return self.to_delete;
    }

    pub fn insert(&mut self, key: &str, val: &impl Serialize) -> serde_json::Result<&mut Self> {
        let mut val = serde_json::to_string(val)?;

        if val.len() > 2 {
            if val.chars().nth(0).unwrap() == '\"' {
                val = val.strip_prefix("\"").unwrap().to_string();
            }
            if val.chars().nth(val.len() - 1).unwrap() == '\"' {
                val = val.strip_suffix("\"").unwrap().to_string();
            }
        }

        self.data.insert(key.to_string(), val);
        self.has_changed = true;
        Ok(self)
    }

    pub fn remove(&mut self, key: &str) -> &mut Self {
        self.data.remove(key);
        self.has_changed = true;
        self
    }

    /// Returns true if session has been changed
    pub fn get_changed(&self) -> bool {
        self.has_changed
    }

    pub fn get_val(&mut self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().gt(&self.expiry.clone())
    }

    pub fn generate_cookie(id: String) -> String {
        BASE64_STANDARD.encode(Sha256::digest(id.into_bytes().as_slice()))
    }

    pub fn set_user_id(&mut self, user_id: Option<String>) -> &mut Self {
        self.user_id = user_id;
        self.has_changed = true;
        self
    }

    pub fn set_user(&mut self, user: &User) -> &mut Self {
        self.set_user_id(Some(user.id.clone()));
        self
    }

    pub fn get_user_id(&self) -> Option<String> {
        return self.user_id.clone();
    }

    pub fn to_db_session(&self) -> DBSession {
        DBSession {
            sess_id: self.clone().id,
            sess_cookie: Session::generate_cookie(self.clone().id),
            expiry: self.expiry.clone().naive_utc(),
            uid: self.user_id.clone(),
            data: Some(serde_json::to_value(self.clone().data).unwrap_or_default()),
        }
    }
}

pub type SessionHandle = Arc<RwLock<Session>>;
