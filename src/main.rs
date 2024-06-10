use reqwest::{
    header::{
        HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL,
        CONNECTION, CONTENT_TYPE, COOKIE, ORIGIN, PRAGMA, REFERER, SET_COOKIE, USER_AGENT,
    },
    StatusCode,
};
use scraper::{Html, Selector};
use std::{collections::HashMap, error::Error, net::IpAddr};
use url::Url;

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

fn detect_asus_rt_ac5300(addr: &IpAddr) -> bool {
    let url = match addr {
        IpAddr::V4(addr) => Url::parse(&format!("http://{}:80/Main_Login.asp", addr)).unwrap(),
        IpAddr::V6(addr) => Url::parse(&format!("http://[{}]:80/Main_Login.asp", addr)).unwrap(),
    };
    let response = reqwest::blocking::get(url).unwrap();
    let body = response.text().unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse(".prod_madelName").unwrap();
    for element in document.select(&selector) {
        if "RT-AC5300" == element.text().collect::<Vec<_>>().concat() {
            return true;
        }
    }
    false
}

fn login_asus_rt_ac5300(addr: &IpAddr, credentials: &String) -> Result<String, Box<dyn Error>> {
    let url = match addr {
        IpAddr::V4(addr) => Url::parse(&format!("http://{}:80/login.cgi", addr)).unwrap(),
        IpAddr::V6(addr) => Url::parse(&format!("http://[{}]:80/login.cgi", addr)).unwrap(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0".parse()?,
    );
    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
            .parse()?,
    );
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.5".parse()?);
    headers.insert(ACCEPT_ENCODING, "gzip, deflate".parse()?);
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse()?);
    headers.insert(ORIGIN, "http://192.168.1.1".parse()?);
    headers.insert(CONNECTION, "keep-alive".parse()?);
    headers.insert(REFERER, "http://192.168.1.1/Main_Login.asp".parse()?);
    headers.insert("Upgrade-Insecure-Requests", "1".parse()?);
    headers.insert("DNT", "1".parse()?);
    headers.insert("Sec-GPC", "1".parse()?);
    headers.insert(PRAGMA, "no-cache".parse()?);
    headers.insert(CACHE_CONTROL, "no-cache".parse()?);

    // Build the form data
    let mut form_data = HashMap::new();
    form_data.insert("group_id", "");
    form_data.insert("action_mode", "");
    form_data.insert("action_script", "");
    form_data.insert("action_wait", "5");
    form_data.insert("current_page", "Main_Login.asp");
    form_data.insert("next_page", "index.asp");
    // base64(username + ":" + password)
    form_data.insert("login_authorization", credentials);
    form_data.insert("login_captcha", "");
    let response = reqwest::blocking::Client::new()
        .post(url)
        .headers(headers.clone())
        .form(&form_data)
        .send()?;
    let resp_headers = response.headers().clone();
    let cookie: Vec<String> = resp_headers
        .get(SET_COOKIE)
        .unwrap()
        .to_str()?
        .split(';')
        .next()
        .unwrap()
        .split('=')
        .map(String::from)
        .collect();
    Ok(cookie.get(1).unwrap().clone())
}

fn enable_port_forwarding_asus_rt_ac5300(
    addr: &IpAddr,
    session: String,
) -> Result<(), Box<dyn Error>> {
    let session = format!("asus_token={session}; clickedItem_tab=0").to_string();
    let url = match addr {
        IpAddr::V4(addr) => Url::parse(&format!("http://{}:80/applyapp.cgi??action_mode=apply&rc_service=restart_firewall&vts_enable_x=1", addr)).unwrap(),
        IpAddr::V6(addr) => Url::parse(&format!("http://[{}]:80/applyapp.cgi??action_mode=apply&rc_service=restart_firewall&vts_enable_x=1", addr)).unwrap(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(COOKIE, session.parse()?);
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_static("1"));
    headers.insert("DNT", HeaderValue::from_static("1"));
    headers.insert("Sec-GPC", HeaderValue::from_static("1"));
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    let response = reqwest::blocking::Client::new()
        .get(url)
        .headers(headers.clone())
        .send()?;
    if response.status() != StatusCode::OK {
        panic!("Failed to enable port forwarding {:?}", response.status());
    }
    Ok(())
}

fn configure_asus_rt_ac5300(addr: &IpAddr, credentials: &String) -> Result<(), Box<dyn Error>> {
    println!("Logging in...");
    let session = login_asus_rt_ac5300(addr, credentials)?;
    println!("Logged in: {session}");
    println!("Enabling port forwarding...");
    enable_port_forwarding_asus_rt_ac5300(addr, session)?;
    println!("Enabled port forwarding!");
    println!("Configured!");
    Ok(())
}

fn main() {
    // base64(username + ":" + password)
    let credentials = String::from("[FILL ME IN]");
    for interface in interfaces() {
        for gateway in interface.gateways {
            if detect_asus_rt_ac5300(&gateway) {
                println!("Detected Asus RT AC5300 @ {gateway}");
                println!("Configuring Asus RT AC5300...");
                configure_asus_rt_ac5300(&gateway, &credentials).unwrap();
            }
        }
    }
}
