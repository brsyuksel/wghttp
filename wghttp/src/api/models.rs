use utoipa::ToSchema;

#[derive(ToSchema)]
pub struct Error {
    pub message: String,
}

#[derive(ToSchema)]
pub struct ListDevice {
    pub device_name: String,
    pub port: u16,
    pub total_peers: u64,
}

#[derive(ToSchema)]
pub struct CreateDevice {
    pub device_name: String,
    pub port: u16,
    pub ip: String,
}

#[derive(ToSchema)]
pub struct DetailDevice {
    pub device_name: String,
    pub port: u16,
    pub ip: String,
    pub public_key: String,
    pub private_key: String,
    pub total_peers: u64,
}

#[derive(ToSchema)]
pub struct ListPeer {
    pub public_key: String,
    pub endpoint: String,
    pub last_handshake_time: u64,
    pub rx: u64,
    pub tx: u64,
    pub persistent_keepalive_time: u16,
    pub allowed_ips: Vec<String>,
}

#[derive(ToSchema)]
pub struct CreatePeer {
    pub persistent_keepalive_time: u16,
    pub allowed_ips: Vec<String>,
}

#[derive(ToSchema)]
pub struct DetailPeer {
    pub public_key: String,
    pub private_key: String,
    pub endpoint: String,
    pub last_handshake_time: u64,
    pub rx: u64,
    pub tx: u64,
    pub persistent_keepalive_time: u16,
    pub allowed_ips: Vec<String>,
}