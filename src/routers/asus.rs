use super::Router;
use anyhow::{anyhow, Result};
use base64::prelude::*;
use cookie::Cookie;
use httparse::{Response, Status};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
};

pub struct Asus {
    gateway: IpAddr,
    descriptor: Option<String>,
    session: Option<String>,
}

const VTS_RULE: &str = "&#60turnitup&#621989&#62192.168.1.250&#621989&#62TCP&#62";

impl Asus {
    pub async fn get_rule_list(&mut self) -> Result<String> {
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let session = self
            .session
            .clone()
            .ok_or(anyhow!("must already be logged in to configure"))?;
        let cookie = format!("Cookie: {}; clickedItem_tab=7", session);
        let params = ["hook=nvram_get(vts_rulelist)"].join("&");
        let firstline = format!("GET /appGet.cgi?{} HTTP/1", params);
        let request = [
            &firstline,
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            &cookie,
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        let payload = payload.as_bytes();
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = Response::new(&mut headers);
        let size = match response.parse(payload)? {
            Status::Complete(size) => size,
            _ => return Err(anyhow!("Expected complete response")),
        };
        let payload = std::str::from_utf8(&payload[size..])?;
        let response = serde_json::from_str::<Value>(payload)?;
        let rules = match response
            .get("vts_rulelist")
            .ok_or(anyhow!("Unable to get existing rules"))?
        {
            Value::String(rules) => rules,
            _ => return Err(anyhow!("unexpected ip address format")),
        };
        Ok(rules.clone())
    }
    pub async fn set_rule(&mut self) -> Result<()> {
        let mut list = self.get_rule_list().await?;
        // Already configured
        if list.contains("turnitup") {
            return Ok(());
        }
        list.push_str(VTS_RULE);
        // Define regex patterns for each custom escape character
        let re_lt = Regex::new(r"&#60").unwrap();
        let re_gt = Regex::new(r"&#62").unwrap();
        let re_backslash = Regex::new(r"&#62192").unwrap();

        // Replace each pattern with the corresponding character
        let result = re_lt.replace_all(&list, "<");
        let result = re_gt.replace_all(&result, ">");
        let result = re_backslash.replace_all(&result, "\\");
        let list = result;
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let session = self
            .session
            .clone()
            .ok_or(anyhow!("must already be logged in to configure"))?;
        let cookie = format!("Cookie: {}; clickedItem_tab=7", session);
        let rulelist = format!("vts_rulelist={}", list);
        let params = [
            "action_mode=apply",
            "rc_service=restart_firewall",
            &rulelist,
        ]
        .join("&");
        let firstline = format!("GET /applyapp.cgi?{} HTTP/1", params);
        let request = [
            &firstline,
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            &cookie,
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        let payload = payload.as_bytes();
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = Response::new(&mut headers);
        let size = match response.parse(payload)? {
            Status::Complete(size) => size,
            _ => return Err(anyhow!("Expected complete response")),
        };
        let payload = std::str::from_utf8(&payload[size..])?;
        serde_json::from_str::<Value>(payload)?;
        Ok(())
    }
    pub async fn get_real_ip(&mut self) -> Result<IpAddr> {
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let session = self
            .session
            .clone()
            .ok_or(anyhow!("must already be logged in to configure"))?;
        let cookie = format!("Cookie: {}; clickedItem_tab=7", session);
        let params = ["hook=nvram_get(wan0_realip_ip)"].join("&");
        let firstline = format!("GET /appGet.cgi?{} HTTP/1", params);
        let request = [
            &firstline,
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            &cookie,
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        let payload = payload.as_bytes();
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = Response::new(&mut headers);
        let size = match response.parse(payload)? {
            Status::Complete(size) => size,
            _ => return Err(anyhow!("Expected complete response")),
        };
        let payload = std::str::from_utf8(&payload[size..])?;
        let response = serde_json::from_str::<Value>(payload)?;
        let ip = match response
            .get("wan0_realip_ip")
            .ok_or(anyhow!("Unable to get ip"))?
        {
            Value::String(ip) => ip,
            _ => return Err(anyhow!("unexpected ip address format")),
        };
        Ok(ip.parse()?)
    }
    async fn enable_port_forwarding(&mut self) -> Result<()> {
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let session = self
            .session
            .clone()
            .ok_or(anyhow!("must already be logged in to configure"))?;
        let cookie = format!("Cookie: {}; clickedItem_tab=7", session);
        let params = [
            "action_mode=apply",
            "rc_service=restart_firewall",
            "vts_enable_x=1",
        ]
        .join("&");
        let firstline = format!("GET /applyapp.cgi?{} HTTP/1", params);
        let request = [
            &firstline,
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            &cookie,
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        let payload = payload.as_bytes();
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = Response::new(&mut headers);
        let size = match response.parse(payload)? {
            Status::Complete(size) => size,
            _ => return Err(anyhow!("Expected complete response")),
        };
        let payload = std::str::from_utf8(&payload[size..])?;
        serde_json::from_str::<Value>(payload)?;
        Ok(())
    }
    async fn forward_port(&mut self, target: IpAddr, port: u16) -> Result<()> {
        Ok(())
    }
    async fn apply(&self) -> Result<()> {
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let session = self
            .session
            .clone()
            .ok_or(anyhow!("must already be logged in to configure"))?;
        let cookie = format!("Cookie: {}; clickedItem_tab=7", session);
        let request = [
            "GET /start_apply.htm HTTP/1",
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            &cookie,
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        Ok(())
    }
}

impl Router for Asus {
    fn new(gateway: IpAddr) -> Self {
        Self {
            gateway,
            descriptor: None,
            session: None,
        }
    }
    async fn probe(&mut self) -> Result<()> {
        // Check if port 80 is open
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        // If port 80 is open, try fetching the login page over HTTP using that port
        let request = [
            "GET /Main_Login.asp HTTP/1",
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Accept: */*",
            "",
            "",
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let document = Html::parse_document(&response);
        let selector = Selector::parse(".prod_madelName").unwrap();
        for element in document.select(&selector) {
            let model = element.text().collect::<Vec<_>>().concat();
            if !model.is_empty() {
                self.descriptor = Some("RT-AC5300".into());
                return Ok(());
            }
        }
        Err(anyhow!("Did not recognize gateway"))
    }
    fn descriptor(&self) -> String {
        match &self.descriptor {
            Some(str) => str.clone(),
            None => panic!("probe must be called before descriptor"),
        }
    }
    async fn login(&mut self, credentials: Vec<String>) -> Result<()> {
        let url = SocketAddr::new(self.gateway, 80);
        let mut stream = TcpStream::connect(url)?;
        let username = credentials.first().ok_or(anyhow!("expected username"))?;
        let password = credentials.get(1).ok_or(anyhow!("expected password"))?;
        let auth = BASE64_STANDARD.encode(format!("{}:{}", username, password));
        let auth = String::from(urlencoding::encode(&auth));
        let auth = format!("login_authorization={}", auth);
        let form_data = [
            "group_id=",
            "action_mode=",
            "action_script=",
            "action_wait=5",
            "current_page=Main_Login.asp",
            "next_page=index.asp",
            &auth,
            "login_captcha=",
        ]
        .join("&");
        let length = format!("Content-Length: {}", form_data.len());
        let request = [
            "POST /login.cgi HTTP/1",
            "HOST: 192.168.1.1",
            "User-Agent: p2p/1.0.0",
            "Referer: http://192.168.1.1/Main_Login.asp",
            "Content-Type: application/x-www-form-urlencoded",
            &length,
            "",
            &form_data,
        ]
        .join("\r\n");
        stream.write_all(request.as_bytes())?;
        let mut payload = String::new();
        stream.read_to_string(&mut payload)?;
        let mut headers = [httparse::EMPTY_HEADER; 64];
        {
            let mut response = Response::new(&mut headers);
            response.parse(payload.as_bytes())?;
        }
        let token = headers
            .iter()
            .find(|header| header.name == "Set-Cookie")
            .ok_or(anyhow!("Did not receive Set-Cookie from router"))?;
        let token = std::str::from_utf8(token.value)?;
        let token = Cookie::parse(token)?;
        self.session = Some(token.encoded().stripped().to_string());
        Ok(())
    }
    async fn configure(&mut self, _ip: IpAddr, _ports: Vec<u16>) -> Result<()> {
        self.enable_port_forwarding().await?;
        self.apply().await?;
        self.set_rule().await?;
        Ok(())
    }
}
