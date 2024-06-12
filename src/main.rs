mod relay;
mod routers;

use anyhow::{anyhow, Result};
use pnet::{
    datalink::{self, NetworkInterface},
    ipnetwork::IpNetwork,
};
use routers::Router;
use std::net::IpAddr;

const PORT: u16 = 1989;

#[derive(Debug)]
struct Address {
    public: IpAddr,
    private: IpAddr,
}

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

async fn asus(interface: &Interface, gateway: &IpAddr) -> Result<Address> {
    let mut asus = routers::asus::Asus::new(*gateway);
    asus.probe().await?;
    println!("Detected {}", asus.descriptor());
    asus.login(vec!["username".into(), "password".into()])
        .await?;
    println!("Logged in");
    let ip = get_ip_address(interface).ok_or(anyhow!("Unable to find ip address"))?;
    asus.configure(*gateway, vec![PORT]).await?;
    println!("Port forwarding configured");
    Ok(Address {
        private: ip,
        public: asus.get_real_ip().await?,
    })
}

async fn router(interface: &Interface) -> Result<Address> {
    for gateway in interface.gateways.iter() {
        match asus(interface, gateway).await {
            Ok(addr) => return Ok(addr),
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }
    }
    Err(anyhow!("Unable to configure router"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut addr: Option<Address> = None;
    for interface in interfaces() {
        match router(&interface).await {
            Ok(ip) => addr = Some(ip),
            Err(e) => println!("{}", e),
        }
    }
    let addr = match addr {
        Some(ip) => ip,
        None => panic!("Unable to configure router"),
    };
    println!("Router for {:?} configured", addr);
    relay::listen(addr.private, PORT);
    Ok(())
}
