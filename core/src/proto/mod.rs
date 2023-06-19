
// use crate::platform::communication::platform::health_check_service_server::{HealthCheckService};
// use platform::health_check_service_server::{HealthCheckService, HealthCheckServiceServer};
// use platform::health_check_service_client::{HealthCheckServiceClient};
// use platform::{HealthCheck, HealthStatus};

// use crate::platformw::communication::platform::health_check_service_server::{SendAlertsService};
// use server_proto::send_alerts_server::{SendAlerts, SendAlertsServer};
// use server_proto::send_alerts_client::{SendAlertsClient};
// use server_proto::{Alert, ReceivedAlerts};
// use uom::si::f32::Time;


pub mod platform {
    tonic::include_proto!("platform");
}

pub mod centre {
    tonic::include_proto!("centre");
}