mod test_helper {
    use std::process::Command;

    pub fn manage_dev<F>(name: &str, fcall: F)
    where
        F: FnOnce() -> (),
    {
        let create_cmd = format!("link add dev {name} type wireguard");
        let create = Command::new("ip")
            .args(create_cmd.split_whitespace())
            .output();
        assert!(create.err().is_none());

        fcall();

        let del_cmd = format!("link del dev {name}");
        let del = Command::new("ip").args(del_cmd.split_whitespace()).output();
        assert!(del.err().is_none());
    }

    pub fn ip_addr_output(name: &str) -> String {
        let cmd = format!("addr show dev {name}");
        let show = Command::new("ip")
            .args(cmd.split_whitespace())
            .output()
            .expect("cant run ip addr cmd");

        std::str::from_utf8(&show.stdout.as_slice())
            .unwrap()
            .to_owned()
    }
}

#[cfg(test)]
mod if_mng_tests {
    use super::{super::InterfaceManagerLibC, test_helper};
    use core::if_mng::*;

    #[test]
    fn fails_on_up_non_existing_device() {
        let m = InterfaceManagerLibC::new("ne0");
        let res = m.up_device().map_err(|e| e.0).err();
        assert_eq!(res, Some("can't get device flags: No such device (os error 19)".to_owned()));
    }

    #[test]
    fn sets_up_device() {
        test_helper::manage_dev("iftest0", || {
            let m = InterfaceManagerLibC::new("iftest0");
            let res = m.up_device().err();
            assert!(res.is_none());

            let ip_addr_res = test_helper::ip_addr_output("iftest0");
            let down = ip_addr_res.contains("DOWN");
            assert!(!down)
        });
    }

    #[test]
    fn fails_setting_ip_and_netmask_non_existing_device() {
        let m = InterfaceManagerLibC::new("ne0");
        let ip = "10.0.0.1".parse().unwrap();
        let netmask = "255.255.255.0".parse().unwrap();
        let res = m.set_ip_and_netmask(&ip, &netmask).map_err(|e| e.0).err();
        assert_eq!(res, Some("can't set ip address: No such device (os error 19)".to_owned()));
    }

    #[test]
    fn sets_ip_and_netmask() {
        test_helper::manage_dev("iftest0", || {
            let m = InterfaceManagerLibC::new("iftest0");
            let ip = "10.0.0.1".parse().unwrap();
            let netmask = "255.255.255.0".parse().unwrap();
            let res = m.set_ip_and_netmask(&ip, &netmask).err();
            assert!(res.is_none());

            let ip_addr_res = test_helper::ip_addr_output("iftest0");
            let ip10001_24 = ip_addr_res.contains("10.0.0.1/24");
            assert!(ip10001_24)
        });
    }
}

#[cfg(test)]
mod device_command_tests {
    use super::{super::DeviceCommand, test_helper};

    #[test]
    fn up_command_fails_non_existing_device() {
        let cmd = DeviceCommand::Up("ne0");
        let res = unsafe { cmd.exec().err() };
        assert_eq!(res, Some("can't get device flags: No such device (os error 19)".to_owned()));
    }

    #[test]
    fn sets_up_device() {
        test_helper::manage_dev("devcmd0", || {
            let cmd = DeviceCommand::Up("devcmd0");
            let res = unsafe { cmd.exec().err() };
            assert!(res.is_none());

            let ip_addr_res = test_helper::ip_addr_output("devcmd0");
            let down = ip_addr_res.contains("DOWN");
            assert!(!down)
        });
    }

    #[test]
    fn fails_setting_ip_non_existing_device() {
        let ip = "10.0.0.1".parse().unwrap();
        let cmd = DeviceCommand::SetIp("ne0", &ip);
        let res = unsafe { cmd.exec().err() };
        assert_eq!(res, Some("can't set ip address: No such device (os error 19)".to_owned()));
    }

    #[test]
    fn sets_ip() {
        test_helper::manage_dev("devcmd1", || {
            let ip = "10.0.0.1".parse().unwrap();
            let cmd = DeviceCommand::SetIp("devcmd1", &ip);
            let res = unsafe { cmd.exec().err() };
            assert!(res.is_none());

            let ip_addr_res = test_helper::ip_addr_output("devcmd1");
            let ip10001_32 = ip_addr_res.contains("10.0.0.1/32");
            assert!(ip10001_32)
        });
    }

    #[test]
    fn fails_setting_netmask_non_existing_device() {
        let netmask = "255.255.255.0".parse().unwrap();
        let cmd = DeviceCommand::SetNetmask("ne0", &netmask);
        let res = unsafe { cmd.exec().err() };
        assert_eq!(res, Some("can't set netmask: No such device (os error 19)".to_owned()));
    }

    #[test]
    fn sets_netmask() {
        test_helper::manage_dev("devcmd2", || {
            let ip = "10.0.0.1".parse().unwrap();
            let netmask = "255.255.255.0".parse().unwrap();
            let ip_cmd = DeviceCommand::SetIp("devcmd2", &ip);
            let netmask_cmd = DeviceCommand::SetNetmask("devcmd2", &netmask);

            let res = unsafe {
                let _ = ip_cmd.exec();
                netmask_cmd.exec().err()
            };

            assert!(res.is_none());

            let ip_addr_res = test_helper::ip_addr_output("devcmd2");
            let ip10001_24 = ip_addr_res.contains("10.0.0.1/24");
            assert!(ip10001_24)
        });
    }
}
