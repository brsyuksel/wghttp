use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListDeviceResponse {
    #[schema(example = "wg0")]
    pub device_name: String,

    #[schema(example = 51820)]
    pub port: u16,

    #[schema(example = 0)]
    pub peers: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeviceIpAddr {
    #[schema(example = "10.0.0.1/24")]
    pub ipv4: Option<String>,

    #[schema(example = "fd86:ea04:1111::1/64")]
    pub ipv6: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDeviceRequest {
    #[schema(example = "wg0")]
    pub device_name: String,

    #[schema(example = 51820)]
    pub port: u16,

    pub ip_addresses: DeviceIpAddr,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDeviceResponse {
    #[schema(example = "wg0")]
    pub device_name: String,

    #[schema(example = 51820)]
    pub port: u16,

    pub ip_addresses: DeviceIpAddr,

    #[schema(example = "UMp441pv9vfOq2eMRK0CURJeSZlsyIDXurczqVKPums=")]
    pub private_key: String,

    #[schema(example = "wfbGOdrEgIGn15y6FMgfJjpaZv02ZQb5xQ5yvnkPhyg=")]
    pub public_key: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DetailDeviceResponse {
    #[schema(example = "wg0")]
    pub device_name: String,

    #[schema(example = 51820)]
    pub port: u16,

    pub ip_addresses: DeviceIpAddr,

    #[schema(example = "wfbGOdrEgIGn15y6FMgfJjpaZv02ZQb5xQ5yvnkPhyg=")]
    pub public_key: String,

    #[schema(example = 0)]
    pub peers: u64,
}
