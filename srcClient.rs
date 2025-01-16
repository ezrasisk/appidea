use tonic::transport::Channel;
use sensor_data::sensor_data_service_client::SensorDataServiceClient;
use sensor_data::{SensorData, TemperatureData};

pub mod sensor_data {
    tonic::include_proto!("sensor_data");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let mut client = SensorDataServiceClient::new(channel);
    let sensor_data = SensorData {
        data: Some(sensor_data::sensor_data::Data::Temperature(TemperatureData {
            sensor_id: "sensor_1".to_string(),
            temperature: 23.5, //or something controlled by user preference 
            timestamp: chrono::Utc::now().timestamp(),
        })),
        auth_token: "your_secret_auth_token".to_string(),
    };

    let response = client.report_sensor_data(sensor_data).await?;
    println!("RESPONSE={:?}", response.into_inner());
    Ok(())
}