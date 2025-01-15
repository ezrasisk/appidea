use influxdb::{Client, WriteQuery};
use chrono::Utc;

#[tokio::main]
async fn write_user_activty(client: &Client, user_id: Uuid, action: &str) -> anyhow::Result<()> {
    let point = WriteQuery::new("user_activity")
        .add_tag("user.id", user_id.to_string())
        .add_field("action", action)
        .timestamp(Utc::now());

    client.query(&point).await?;
    Ok(())
}

pub async fn batch_write_user_activities(client: &Client, activities: Vec<(Uuid, String)>) -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = Vec::new();
    for (user_id, action) in activities{
        batch.push(WriteQuery::new("user_activity").add_tag("user_id", user_id.to_string()).add_field("action", action).timestamp(Utc::now()));
    }
    client.query_batch(&batch).await?;
    Ok(())
}

#[derive(Debug)]
enum UserError {
    InvalidUUID,
    InvalidDateTime,
}

impl From<chrono::ParseError> for UserError {
    fn from(_: chrono::ParseError) -> Self {
        UserError::InvalidDateTime
    }
}

//may belong in user structure, not too sure