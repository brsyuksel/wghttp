pub mod wg {
    #[derive(Debug)]
    pub struct WGError(pub String);

    impl std::fmt::Display for WGError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "WGError: {}", self.0)
        }
    }

    impl std::error::Error for WGError {}

    #[derive(Debug)]
    pub struct WGDevice {
        pub name: String,
        pub public_key: String,
        pub private_key: String,
        pub port: u16,
        pub peers: u64,
    }

    #[derive(Debug)]
    pub struct WGPeer {
        pub allowed_ips: Vec<String>,
        pub endpoint: String,

        pub last_handshake_time: i64,
        pub persistent_keepalive_interval: u16,

        pub rx: u64,
        pub tx: u64,

        pub public_key: String,
        pub private_key: String,
        pub preshared_key: String,
    }
}

pub mod netdev {
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[derive(Debug)]
    pub struct NetDevError(pub String);

    impl std::fmt::Display for NetDevError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "NetDevError: {}", self.0)
        }
    }

    impl std::error::Error for NetDevError {}

    #[derive(Debug)]
    pub struct NetDevIp {
        pub ipv4: Option<(Ipv4Addr, u8)>,
        pub ipv6: Option<(Ipv6Addr, u8)>,
    }
}
