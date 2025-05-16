use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ListPeerResponse {
    #[schema(example = "wfbGOdrEgIGn15y6FMgfJjpaZv02ZQb5xQ5yvnkPhyg=")]
    pub public_key: String,

    #[schema(example = "15.16.17.18:4321")]
    pub endpoint: String,

    #[schema(example = json!(["10.0.0.2/32", "fd86:ea04:1111::2/128"]))]
    pub allowed_ips: Vec<String>,

    #[schema(example = 1745760960)]
    pub last_handshake_time: i64,

    #[schema(example = 30)]
    pub persistent_keepalive_interval: u16,

    #[schema(example = 4582376)]
    pub rx: u64,

    #[schema(example = 7231842)]
    pub tx: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreatePeerRequest {
    #[schema(example = json!(["10.0.0.2/32", "fd86:ea04:1111::2/128"]))]
    pub allowed_ips: Vec<String>,

    #[schema(example = 30)]
    pub persistent_keepalive_interval: u16,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreatePeerResponse {
    #[schema(example = "wfbGOdrEgIGn15y6FMgfJjpaZv02ZQb5xQ5yvnkPhyg=")]
    pub public_key: String,

    #[schema(example = "UMp441pv9vfOq2eMRK0CURJeSZlsyIDXurczqVKPums=")]
    pub private_key: String,

    #[schema(example = "GBVavxe7VEId8K9/trxquNihyEES3p9ydJ2pWQVI5j0=")]
    pub preshared_key: String,

    #[schema(example = json!(["10.0.0.2/32", "fd86:ea04:1111::2/128"]))]
    pub allowed_ips: Vec<String>,

    #[schema(example = 30)]
    pub persistent_keepalive_interval: u16,
}
