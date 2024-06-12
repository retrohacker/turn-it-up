pub mod asus;

use anyhow::Result;
use std::net::IpAddr;

pub trait Router {
    fn new(gateway: IpAddr) -> Self;
    fn descriptor(&self) -> String;
    async fn probe(&mut self) -> Result<()>;
    async fn login(&mut self, credentials: Vec<String>) -> Result<()>;
    async fn configure(&mut self, ip: IpAddr, ports: Vec<u8>) -> Result<()>;
}
