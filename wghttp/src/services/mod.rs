use std::sync::Arc;
use domain::adapters::wg::WireguardAdapter;

#[derive(Clone)]
pub struct TunnelManager {
    pub wireguard: Arc<dyn WireguardAdapter>,
}

impl TunnelManager {
    pub fn new<W: WireguardAdapter + 'static>(wg_adapter: W) -> Self {
        let wg_arc = Arc::new(wg_adapter);

        TunnelManager { wireguard: wg_arc }
    }
}
