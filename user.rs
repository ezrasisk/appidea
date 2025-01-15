use chrono::{DataTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tonic::{include_proto, Status};
use bcrypt::{hash, verify, DEFAULT_COST};
use validator::Validate;
use std::sync::Arc;
use tokio::sync::Mutex;

include_proto!("user");

pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DataTime<Utc>,
    pub roles: Vec<String>,
    pub is_active: bool,
}

pub enum UserEvent {
    UserCreated {
        user_id: Uuid,
        username: String,
        email: String,
        created_at: DateTime<Utc>,
    },
    UserUpdated {
        user_id: Uuid,
        username: Option<String>,
        email: Option<String>,
        updated_at: DateTime<Utc>,
    },
    //more event types as needed

pub struct RateLimiter {
    attempts: Arc<MutexMap<Uuid, Vec<DateTime<Utc>>>>
}

impl RateLimiter {
    pub async fn check(&self, user_id: Uuid) -> bool {
        let now = Utc::now();
        let mut attempts = self.attempts.lock().await;
        let user_attempts = attempts.entry(user_id).or_insert_with(Vec::new);

        user_attempts.retain(|&t| now.signed_duration_since(t).num_seconds() < 60);

        if user_attempts.len() >= 5 {
            false
        } else {
            user_attempts.push(now);
            true
        }
    }
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            username,
            email,
            created_at: now,
            roles: vec!["user".to_string()],
            is_active: true,
        }
    }

    pub fn to_proto(&self) -> user::User {
        user::User {
            id: self.id.to_string(),
            username:self.username.clone(),
            email: self.email.clone(),
            created_at: self.created_at.to_rfc3339(),
            roles: self.roles.clone(),
            is_active: self.is_active,
        }
    }

    pub fn from_proto(user: user::User) -> Result<Self, Status> {
        let id = Uuid::parse_str(&user.id).map_err(|_| Status::invlalid_argument("Invalid UUID"))?;
        let created_at = DataTime::parse_from_rfc3339(&user.created_at).map_err(|_| Status::invlalid_argument("Invalid datetime"))?.with_timezone(&Utc);
        Ok(User {
            id,
            username: user.username,
            email: user.email,
            created_at,
            roles: user.roles,
            is_active: user.is_active,
        })
    }

    pub fn hash_password(&mut self, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let hash = hash(password, DEFAULT_COST)?;
        self.password_hash = hash;
        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, Box<dyn std::error::Error>> {
        verify(password, &self.password_hash)
    }

    pub fn from_proto(user: user::User) -> Result<Self, UserError> {
        let id = Uuid::parse_str(&user.id).map_err(|_| UserError::InvalidUUID)?;
        let created_at = DataTime::parse_from_rfc3339(&user.created_at).map_err(|_| UserError::InvalidDateTime)?.with_timezone(&Utc);
        Ok(User {
            id,
            username: user.username,
            email: user.email,
            created_at,
            roles: user.roles,
            is_active: user.is_active,
        })
        //handles all errors, possibly better than other from_proto
    }

    pub enum UserServiceError {
        ValidationError(ValidationError), DBERROR(String), //other errors...
    }

    pub From<UserServiceError> for Status {
        fn from(error: UserServiceError) -> Self {
            match error{
                UserServiceError::ValidationError(_) => Status::invlalid_argument("Validation error"), UserServiceError::DBERROR(_) => Status::internal("Database error"), //other mappings
            }
        }
    }

    pub fn validate(&self) -> Result<(), validator::ValidationError> {
        self.validate()
        //this or that !
    }

    fn validate(&self) -> Result<(), ValidationError>{
        //email validation, user uniqueness, username user differentiability, fn or pub fn?^^^
    }

    async fn get_last_login(&self, client: &Client) -> Result<DataTime<Utc>, Box<dyn std::error::Error>> {
        //query for last login time from user, possibly already in other project folder
    }

    async fn store_event(client: &Client, event: &UserEvent) -> Result<(), Box<dyn std::error::Error>> {
        let event_type = format!("{:?", event);
        let point = WriteQuery::new("user_events").add_tag("event_type", event_type).add_field("payload", serde_json::to_string(event)?).timestamp(Utc::now());
        client.query(&point).await?;
        Ok(())

}

impl UserAggregate {
    pub fn new() -> Self {
        UserAggregate {
            id: Uuid::new_v4(),
            username: String::new(),
            email: String::new(),
            roles: Vec::new(),
            is_active: true,
            version: 0,
        }
    }

    pub fn apply_event(&mut self, event: UserEvent) {
        match event {
            UserEvent::UserCreated { user_id, username, email, .. } => {
                self.id = user_id;
                self.username = username;
                self.email = email;
                self.version += 1;
            } UserEvent::UserUpdated {user_id, username, email, .. } => {
                if let Some(new_username) = username { self.username = new_username; }
                if let Some(new_email) = email { self.email = new_email; } self.version += 1;}
            //other events to handle
        }
    }
}

async fn load_user(client: &Client, user_id: Uuid) -> Result<UserAggregate, Box<dyn std::error::Error>> {
    let mut user = UserAggregate::new();
    let query = "SELECT * FROM user_events WHERE \"user_id\" = $1 ORDER BY time";
    let events = client.query(query, &[&user_id.to_string()]).await?;
    for evet in events.iter() {
        if let Some(payload) = event.fields.get("payload") {
            if let Ok(evt) = serde_json::from_str::UserEvent>(pay-load.as_str().unwrap()) {
                user.apply_event(evt);
            }
        }
    }
    Ok(user)
}

async fn handle_create_user(user_id: Uuid, username: String, email: String) -> Result<(), Box<dyn std::error::Error>> {
    let event = UserEvent::UserCreated { user_id, username, email, created_at:Utc::now() };
    store_event(&client, &event).await?;
    Ok(())
}

impl From<User> for user::User {
    fn from(user: User) -> Self {
        user.to_proto()
    }
}

impl TryFrom<user::User> for User {
    type Error = Status;
    fn try_from(user: user::User) -> Result<Self, Self::Error> {
        User::from_proto(user)
    }
}

impl From<chrono::ParseError> for UserError {
    fn from(_: chrono::ParseError) -> Self {
        UserError::InvalidDateTime
    }
}
