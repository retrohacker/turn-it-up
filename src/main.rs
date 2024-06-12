mod relay;
mod routers;

use anyhow::{anyhow, Result};
use pnet::{
    datalink::{self, NetworkInterface},
    ipnetwork::IpNetwork,
};
use routers::Router;
use std::net::IpAddr;

#[derive(Debug)]
struct Interface {
    name: String,
    ips: Vec<IpAddr>,
    gateways: Vec<IpAddr>,
}

fn interfaces() -> Vec<Interface> {
    let mut result = Vec::new();
    for interface in netdev::get_interfaces() {
        println!("{:?}", interface);
        let ips = Vec::new();
        match &interface.gateway {
            Some(gateway) => {
                println!("{:?}", gateway);
                let mut gateways: Vec<IpAddr> =
                    Vec::with_capacity(gateway.ipv4.len() + gateway.ipv6.len());
                let ipv4: Vec<IpAddr> = gateway.ipv4.clone().into_iter().map(IpAddr::V4).collect();
                gateways.extend(ipv4);
                result.push(Interface {
                    name: interface.name,
                    gateways,
                    ips: ips.clone(),
                })
            }
            None => {}
        }
    }
    result
}

fn get_ip_address(interface: &Interface) -> Option<IpAddr> {
    // Find the network interface by name and get its IP address
    datalink::interfaces()
        .into_iter()
        .find(|candidate| candidate.name == interface.name)
        .and_then(|candidate| {
            candidate.ips.into_iter().find_map(|ip| match ip {
                IpNetwork::V4(ipv4) => Some(ipv4.ip().into()),
                _ => None,
            })
        })
}

async fn asus(gateway: &IpAddr) -> Result<()> {
    let mut asus = routers::asus::Asus::new(*gateway);
    asus.probe().await?;
    println!("Detected {}", asus.descriptor());
    asus.login(vec![
        "retrohacker".into(),
        "ShortageSlashedSmithGlancing".into(),
    ])
    .await?;
    println!("Logged in");
    // Todo: set our IP address and desired ports
    asus.configure(*gateway, Vec::new()).await?;
    println!("Port forwarding configured");
    Ok(())
}

async fn router(interface: &Interface) -> Result<IpAddr> {
    for gateway in interface.gateways.iter() {
        if let Err(_) = asus(gateway).await {
            continue;
        }
        let ip = get_ip_address(interface).ok_or(anyhow!("Unable to find ip address"))?;
        return Ok(ip);
    }
    Err(anyhow!("Unable to configure router"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut addr: Option<IpAddr> = None;
    for interface in interfaces() {
        if let Ok(ip) = router(&interface).await {
            addr = Some(ip);
        }
    }
    let addr = match addr {
        Some(ip) => ip,
        None => panic!("Unable to configure router"),
    };
    println!("Router for {} configured", addr);
    // Todo: discover public ip address
    // Todo: configure libp2p relay server
    // Todo: confirm we are routable
    Ok(())
}
