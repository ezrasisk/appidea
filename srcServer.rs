use tonic::{transport::Server, Request, Response, Status};
use sensor_data::sensor_data_service_server::{SensorDataService, SensorDataServiceServer};
use sensor_data::{SensorData, DataResponse};
use influxdb::{Client, WriteQuery};
use anyhow::Result;
use chrono::{DateTime,Utc};
use uuid::Uuid;

pub mod sensor_data {
    tonic::include_proto!("sensor_data");
}

#[derive(Debug)]
pub struct MySensorDataService {
    influx_client: Client,
}

impl MySensorDataService {
    pub async fn new(influx_url: &str) -> Result<Self> {
        let influx_client = Client::new(influx_url, "your_db_name");
        Ok(MySensorDataService { influx_client
        })
    }
    async fn store_data(&self, data: &SensorData) -> Result<()> {
        let measurement = match data.data.as_ref(){
            Some(sensor_data::sensor_data::Data::Temperature(temp)) => {
                WriteQuery::new("temperature").add_tag("sensor_id", &temp.sensor_id).add_field("value", temp.temperature).timestamp(DataTime::<Utc>::from_timestamp(temp.timestamp, 0).unwrap())
            }

            Some(sensor_data::sensor_data::Data::Humidity(hum)) => {
                WriteQuery::new("humidity").add_tag("sensor_id", &hum.sensor_id).add_field("value", hum.humidity).timestamp(DataTime::<Utc>::from_timestamp(hum.timestamp, 0).unwrap())
            }
            
            None => return Err(anyhow::anyhow!("No data provided")),
        };

        self.influx_client.query(&measurement).await?;
        Ok(())
    }

    fn validate_auth(&self, auth_token: &str) -> Result<(), Status> {
        if auth_token != "your_secret_auth_token" {
            return Err(Status::unauthenticated("Invalid auth token")); //kaspa token verification imp later
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl SensorDataService for MySensorDataService {
    async fn report_sensor_data(&self, request: Request<SensorData>) -> Result<Response<DataResponse>, Status> {
        let sensor_data = request.into_inner();
        self.validate_auth(&sensor_data.auth_token)?;
        match
        self.store_data(&sensor_data).await {
            Ok(_) => Ok(Response::new(DataResponse {
                success: true,
                message: "Data recieved and stored successfully".to_string(),
            })),
            Err(e) => {
                eprintln!("Error storing data: {:?}", e);
                Err(Status::internal("Failed to store data"))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let sensor_data_service = MySensorDataService::new("heep://localhost:8086").await?

    Server::builder().add_service(SensorDataServiceServer::new(sensor_data_service)).serve(addr).await?;
    Ok(())
}