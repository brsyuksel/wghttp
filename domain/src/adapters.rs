pub mod wg {
    use crate::models::wg::*;

    /// Adapter interface for interacting with WireGuard devices and peers.
    ///
    /// Implementations of this trait are responsible for managing WireGuard state,
    /// including device lifecycle and peer configuration.
    pub trait WireguardAdapter: Send + Sync {
        /// Retrieves information about a WireGuard device by name.
        ///
        /// Returns a WGDevice struct with metadata about the interface.
        fn get_device(&self, device_name: &str) -> Result<WGDevice, WGError>;

        /// Lists all available WireGuard devices currently configured on the system.
        ///
        /// Returns a Vec of WGDevice.
        fn list_devices(&self) -> Result<Vec<WGDevice>, WGError>;

        /// Creates a new WireGuard device with the given name and port.
        ///
        /// On success, returns the created WGDevice instance.
        fn create_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WGError>;

        /// Deletes the WireGuard device with the specified name.
        ///
        /// Returns Ok(()) if the device was successfully deleted.
        fn delete_device(&self, device_name: &str) -> Result<(), WGError>;

        /// Lists all peers configured on the specified WireGuard device.
        ///
        /// Returns a Vec of WGPeer structs describing each peer.
        fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WGError>;

        /// Adds a peer to the specified device with the given allowed IPs and keepalive.
        ///
        /// Returns the created WGPeer instance.
        fn add_peer(
            &self,
            device_name: &str,
            allowed_ips: Vec<&str>,
            persistent_keepalive_interval: u16,
        ) -> Result<WGPeer, WGError>;

        /// Deletes a peer from the device using its public key.
        ///
        /// Returns Ok(()) if the peer was successfully removed.
        fn delete_peer(&self, device_name: &str, public_key: &str) -> Result<(), WGError>;
    }
}
