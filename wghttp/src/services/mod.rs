use domain::adapters::netdev::NetworkDeviceAdapter;
use domain::adapters::wg::WireguardAdapter;
use std::sync::Arc;

#[derive(Clone)]
pub struct TunnelManager {
    pub wireguard: Arc<dyn WireguardAdapter>,
    pub netdev: Arc<dyn NetworkDeviceAdapter>,
}

impl TunnelManager {
    pub fn new<W, N>(wg_adapter: W, netdev_adapter: N) -> Self
    where
        W: WireguardAdapter + 'static,
        N: NetworkDeviceAdapter + 'static,
    {
        let wg_arc = Arc::new(wg_adapter);
        let nd_arc = Arc::new(netdev_adapter);

        TunnelManager {
            wireguard: wg_arc,
            netdev: nd_arc,
        }
    }
}
