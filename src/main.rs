mod routers;

use anyhow::Result;
use routers::Router;
use std::net::IpAddr;

#[derive(Debug)]
struct Interface {
    gateways: Vec<IpAddr>,
}

fn interfaces() -> Vec<Interface> {
    let mut result = Vec::new();
    for interface in netdev::get_interfaces() {
        match &interface.gateway {
            Some(gateway) => {
                let mut gateways: Vec<IpAddr> =
                    Vec::with_capacity(gateway.ipv4.len() + gateway.ipv6.len());
                let ipv4: Vec<IpAddr> = gateway.ipv4.clone().into_iter().map(IpAddr::V4).collect();
                let ipv6: Vec<IpAddr> = gateway.ipv6.clone().into_iter().map(IpAddr::V6).collect();
                gateways.extend(ipv4);
                gateways.extend(ipv6);
                result.push(Interface { gateways })
            }
            None => {}
        }
    }
    result
}

async fn asus(gateway: &IpAddr) -> Result<()> {
    let mut asus = routers::asus::Asus::new(*gateway);
    asus.probe().await?;
    println!("Detected {}", asus.descriptor());
    asus.login(vec!["username".into(), "password".into()])
        .await?;
    println!("Logged in");
    // Todo: set our IP address and desired ports
    asus.configure(*gateway, Vec::new()).await?;
    println!("Port forwarding configured");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    for interface in interfaces() {
        for gateway in interface.gateways {
            asus(&gateway).await?;
        }
    }
    // Todo: remember state from previous runs
    // Todo: configure libp2p relay server
    Ok(())
}
