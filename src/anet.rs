use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

pub fn resolve_host(host: &str) -> Result<String, String> {
    if let Ok(ip) = IpAddr::from_str(host) {
        return Ok(ip.to_string());
    }

    let target = format!("{}:1", host);
    match target.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                Ok(addr.ip().to_string())
            } else {
                Err(format!("cannot resolve host: {}", host))
            }
        }
        Err(_) => Err(format!("cannot resolve host: {}", host)),
    }
}

pub fn tcp_generic_connect(addr: &str, port: u16, non_blocking: bool) -> Result<TcpStream, String> {
    let addrs = format!("{}:{}", addr, port).to_socket_addrs().map_err(|e| e.to_string())?;
    let stream = TcpStream::connect_timeout(&addrs.into_iter().next().ok_or("No valid addr")?, Duration::from_secs(1))
        .map_err(|e| e.to_string())?;

    if non_blocking {
        stream
            .set_nonblocking(true)
            .map_err(|e| format!("set non-blocking error: {}", e))?;
    }

    Ok(stream)
}

pub fn tcp_connect(addr: &str, port: u16) -> Result<TcpStream, String> {
    tcp_generic_connect(addr, port, false)
}

pub fn tcp_non_block_connect(addr: &str, port: u16) -> Result<TcpStream, String> {
    tcp_generic_connect(addr, port, true)
}
