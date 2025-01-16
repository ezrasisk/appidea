//make sure these are in the files head
use tonic::{Request, Response, Status};
use user::user_service::UserService;
use user::{User as UserProto, UserResponse};

pub struct MyUserService {
    client: Client,
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn created_user(&self, request: Request<UserProto>) -> Result<Response<UserResponse>, Status> {
        let user_proto = request.into_inner();
        let user_id = create_user(&self.client, user_proto.username, user_proto.email).await.map_err(|_| Status::internal("Failed to create user"))?;
        Ok(Response::new(UserResponse {
            success: true,
            message: format!("User created with id: {}", user_id),
        }))
    }
    //other methods
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSnapshot {
    pub id: Uuid,
    pub username: String,
    pub email: String, 
    pub version: u32 //number of preprocessed events
}

async fn create_snapshot(client: &Client, user: &User) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = UserSnapshot {
        id: user.id,
        username: user.username.clone(),
        email: user.email.clone(),
        version: user.version, //needs user version field to catalogue applied events
    };

    let point = WriteQuery::new("user_snapshots").add_field("payload", serde_json::to_string(&snapshot)?).timestamp(Utc::now());
    client.query(&point).await?;
    Ok(())
}

async fn load_user_with_snapshots(client: &Client, user_id:Uuid) -> Result<User, Box<dyn std::error::Error>> {
    let mut user = User::new(user_id, String::new(), String::new(), 0); //with user version field

    let snapshot_query = "SELECT * FROM user_snapshots WHERE \"id\" = $1 ORDER BY time DESC LIMIT 1";
    let snapshots = client.query(snapshot_query, &[&user_id.tostring()]).await?;
    if let Some(snapshot) = snapshot.fields.get("payload") {
        if let Ok(snap) = serde_json::from_str::<UserSnapshot>(payload-.as_str().unwrap()) {
            user.id = snap.id;
            user.username = snap.username;
            user.email = snap.email;
            user.version = snap.version;
        }
    }


    let event_query = "SELECT * FRM user_events WHERE \"id\" = $1 AND time > $2 ORDER BY time";
    let events = client.query(event_query, &[&user_id.to_string(), &snapshots[0].time]).await?;
    for event in events.iter() {
        if let Some(payload) = event.fields.get("payload") {
            if let Ok(evt) = serde_json::from_str::<UserEvent>(payload-.as_str().unwrap()) {
                user.apply_event(evt);
            }
        }
    }
    Ok(user)
}
