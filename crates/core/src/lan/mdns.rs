use super::protocol::{get_local_ip_addresses, DiscoveredHost, DiscoveredHostList, LanError};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct MdnsBroadcaster {
    daemon: ServiceDaemon,
    fullname: String,
}

impl MdnsBroadcaster {
    pub fn new(
        institution_name: &str,
        host_windows_name: &str,
        host_id: &str,
        fingerprint: &str,
        port: u16,
        pairing_open: bool,
    ) -> Result<Self, LanError> {
        let mdns = ServiceDaemon::new().map_err(|_| LanError::DiscoveryTimeout)?;
        let service_type = "_coachify._tcp.local.";
        let instance_name = format!("Coachify - {} ({})", institution_name, host_windows_name);
        let host_name = format!("{}.local.", host_windows_name.replace(' ', "-"));

        let mut properties = HashMap::new();
        properties.insert("host_id".to_string(), host_id.to_string());
        properties.insert("proto".to_string(), "1".to_string());
        properties.insert("fingerprint".to_string(), fingerprint.to_string());
        properties.insert("inst".to_string(), institution_name.to_string());
        properties.insert("ver".to_string(), "0.1.0".to_string());
        properties.insert(
            "pairing".to_string(),
            if pairing_open { "1" } else { "0" }.to_string(),
        );

        let primary_ip = get_local_ip_addresses()
            .first()
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());

        let service_info = ServiceInfo::new(
            service_type,
            &instance_name,
            &host_name,
            primary_ip.as_str(),
            port,
            properties,
        )
        .map_err(|e| LanError::Database(format!("mDNS ServiceInfo error: {}", e)))?;

        let fullname = service_info.get_fullname().to_string();
        mdns.register(service_info)
            .map_err(|e| LanError::Database(format!("mDNS register error: {}", e)))?;

        Ok(Self {
            daemon: mdns,
            fullname,
        })
    }

    pub fn update_pairing_status(
        &mut self,
        institution_name: &str,
        host_windows_name: &str,
        host_id: &str,
        fingerprint: &str,
        port: u16,
        pairing_open: bool,
    ) {
        let service_type = "_coachify._tcp.local.";
        let instance_name = format!("Coachify - {} ({})", institution_name, host_windows_name);
        let host_name = format!("{}.local.", host_windows_name.replace(' ', "-"));

        let mut properties = HashMap::new();
        properties.insert("host_id".to_string(), host_id.to_string());
        properties.insert("proto".to_string(), "1".to_string());
        properties.insert("fingerprint".to_string(), fingerprint.to_string());
        properties.insert("inst".to_string(), institution_name.to_string());
        properties.insert("ver".to_string(), "0.1.0".to_string());
        properties.insert(
            "pairing".to_string(),
            if pairing_open { "1" } else { "0" }.to_string(),
        );

        let primary_ip = get_local_ip_addresses()
            .first()
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_string());

        if let Ok(service_info) = ServiceInfo::new(
            service_type,
            &instance_name,
            &host_name,
            primary_ip.as_str(),
            port,
            properties,
        ) {
            let _ = self.daemon.register(service_info);
        }
    }

    pub fn shutdown(self) {
        let _ = self.daemon.unregister(&self.fullname);
        let _ = self.daemon.shutdown();
    }
}

pub fn discover_hosts(timeout_ms: Option<u64>) -> Result<DiscoveredHostList, LanError> {
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(2000));
    let mdns = ServiceDaemon::new().map_err(|_e| LanError::DiscoveryTimeout)?;
    let receiver = mdns
        .browse("_coachify._tcp.local.")
        .map_err(|_| LanError::DiscoveryTimeout)?;

    let start = Instant::now();
    let mut discovered: HashMap<String, DiscoveredHost> = HashMap::new();

    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        if let Ok(ServiceEvent::ServiceResolved(info)) =
            receiver.recv_timeout(remaining.min(Duration::from_millis(300)))
        {
            let host_id = info
                .get_property_val_str("host_id")
                .unwrap_or("unknown")
                .to_string();
            let fingerprint = info
                .get_property_val_str("fingerprint")
                .unwrap_or("")
                .to_string();
            let app_ver = info
                .get_property_val_str("ver")
                .unwrap_or("0.1.0")
                .to_string();
            let name = info
                .get_property_val_str("inst")
                .unwrap_or(info.get_fullname())
                .to_string();

            let ip = info
                .get_addresses_v4()
                .iter()
                .next()
                .map(|ip| ip.to_string())
                .or_else(|| info.get_addresses().iter().next().map(|ip| ip.to_string()))
                .unwrap_or_else(|| "127.0.0.1".to_string());

            let host = DiscoveredHost {
                host_id: host_id.clone(),
                name,
                ip,
                port: info.get_port(),
                fingerprint,
                app_version: app_ver,
            };
            discovered.insert(host_id, host);
        }
    }

    let _ = mdns.shutdown();
    Ok(DiscoveredHostList {
        hosts: discovered.into_values().collect(),
    })
}
