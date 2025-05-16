use std::net::IpAddr;
use std::str::FromStr;

pub const DEVICE_NAME_MAX_LEN: usize = 15;
pub const PUBKEY_MAX_LEN: usize = 44;

pub fn parse_ip(input: &str) -> Result<(IpAddr, u8), String> {
    let (ip_str, prefix_str_opt) = input
        .split_once('/')
        .map(|(ip, prefix)| (ip, Some(prefix)))
        .unwrap_or((input, None));

    let ip = IpAddr::from_str(ip_str).map_err(|_| format!("invalid ip address: {}", ip_str))?;

    let max_prefix = match ip {
        IpAddr::V4(_) => 32,
        IpAddr::V6(_) => 128,
    };

    let prefix = match prefix_str_opt {
        None => max_prefix,
        Some(p) => p
            .parse::<u8>()
            .map_err(|_| format!("invalid prefix: {}", p))?,
    };

    if prefix > max_prefix {
        return Err(format!("prefix too large: {} (max {})", prefix, max_prefix));
    }

    Ok((ip, prefix))
}

pub fn validate_ip_list(ip_list: &[String]) -> Result<(), String> {
    for ip_str in ip_list {
        parse_ip(ip_str)?;
    }

    Ok(())
}
