use crate::IpClass;
use hickory_proto::op::Message;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use thiserror::Error;
use tokio::net::UdpSocket;

pub const MDNS_PORT: u16 = 5353;
pub const MDNS_IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const MDNS_IPV6: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0xfb);

#[derive(Debug)]
#[doc(hidden)]
pub enum IP {
    Ipv4,
    Ipv6,
}

impl std::fmt::Display for IP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IP::Ipv4 => write!(f, "IPv4"),
            IP::Ipv6 => write!(f, "IPv6"),
        }
    }
}

/// Errors that can occur when creating and configuring an IPv4 or IPv6 socket.
#[derive(Debug, Error)]
pub enum SocketError {
    #[error("{domain}: error creating new socket for")]
    NewSocket {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error setting the reuse address")]
    ReuseAddress {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[cfg(unix)]
    #[error("{domain}: error setting the reuse port")]
    ReusePort {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error binding the socket for")]
    Bind {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error setting multicast loop")]
    SetMulticastLoop {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error joining multicast")]
    JoinMulticast {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error setting the multicast ttl")]
    MulticastTtl {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error setting the socket to non-blocking mode")]
    SetNonBlocking {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("{domain}: error creating a UDP socket from a standard socket")]
    UdpSocket {
        domain: IP,
        #[source]
        source: std::io::Error,
    },
    #[error("Cannot bind to IPv4 or IPv6")]
    CannotBind,
}

pub fn socket_v4() -> Result<UdpSocket, SocketError> {
    socket_v4_on_interface(Ipv4Addr::UNSPECIFIED)
}

/// Create an IPv4 mDNS socket bound to a specific interface.
/// 
/// The `interface` parameter should be the IPv4 address of the interface to bind to.
/// Use `Ipv4Addr::UNSPECIFIED` (0.0.0.0) to bind to all interfaces.
pub fn socket_v4_on_interface(interface: Ipv4Addr) -> Result<UdpSocket, SocketError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).map_err(|source| {
        SocketError::NewSocket {
            domain: IP::Ipv4,
            source,
        }
    })?;
    socket
        .set_reuse_address(true)
        .map_err(|source| SocketError::ReuseAddress {
            domain: IP::Ipv4,
            source,
        })?;
    #[cfg(unix)]
    socket
        .set_reuse_port(true)
        .map_err(|source| SocketError::ReusePort {
            domain: IP::Ipv4,
            source,
        })?;
    socket
        .bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MDNS_PORT).into())
        .map_err(|source| SocketError::Bind {
            domain: IP::Ipv4,
            source,
        })?;
    socket
        .set_multicast_loop_v4(true)
        .map_err(|source| SocketError::SetMulticastLoop {
            domain: IP::Ipv4,
            source,
        })?;
    socket
        .join_multicast_v4(&MDNS_IPV4, &interface)
        .map_err(|source| SocketError::JoinMulticast {
            domain: IP::Ipv4,
            source,
        })?;
    socket
        .set_multicast_ttl_v4(16)
        .map_err(|source| SocketError::MulticastTtl {
            domain: IP::Ipv4,
            source,
        })?;
    socket
        .set_nonblocking(true)
        .map_err(|source| SocketError::SetNonBlocking {
            domain: IP::Ipv4,
            source,
        })?;
    UdpSocket::from_std(std::net::UdpSocket::from(socket)).map_err(|source| {
        SocketError::UdpSocket {
            domain: IP::Ipv4,
            source,
        }
    })
}

pub fn socket_v6() -> Result<UdpSocket, SocketError> {
    socket_v6_on_interface(0)
}

/// Create an IPv6 mDNS socket bound to a specific interface.
/// 
/// The `interface_index` parameter should be the index of the interface to use for multicast.
/// Use 0 to let the system choose the default interface.
pub fn socket_v6_on_interface(interface_index: u32) -> Result<UdpSocket, SocketError> {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).map_err(|source| {
        SocketError::NewSocket {
            domain: IP::Ipv6,
            source,
        }
    })?;
    socket
        .set_reuse_address(true)
        .map_err(|source| SocketError::ReuseAddress {
            domain: IP::Ipv6,
            source,
        })?;
    #[cfg(unix)]
    socket
        .set_reuse_port(true)
        .map_err(|source| SocketError::ReusePort {
            domain: IP::Ipv6,
            source,
        })?;
    socket
        .bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, MDNS_PORT)).into())
        .map_err(|source| SocketError::Bind {
            domain: IP::Ipv6,
            source,
        })?;
    socket
        .set_multicast_loop_v6(true)
        .map_err(|source| SocketError::SetMulticastLoop {
            domain: IP::Ipv6,
            source,
        })?;
    socket
        .join_multicast_v6(&MDNS_IPV6, interface_index)
        .map_err(|source| SocketError::JoinMulticast {
            domain: IP::Ipv6,
            source,
        })?;
    socket
        .set_nonblocking(true)
        .map_err(|source| SocketError::SetNonBlocking {
            domain: IP::Ipv6,
            source,
        })?;
    UdpSocket::from_std(std::net::UdpSocket::from(socket)).map_err(|source| {
        SocketError::UdpSocket {
            domain: IP::Ipv6,
            source,
        }
    })
}

#[derive(Clone, Debug)]
pub struct Sockets {
    v4: Option<Arc<UdpSocket>>,
    v6: Option<Arc<UdpSocket>>,
}

impl Sockets {
    pub fn new(class: IpClass) -> Result<Self, SocketError> {
        match class {
            IpClass::Auto => {
                let socket = Self {
                    v4: socket_v4().ok().map(Arc::new),
                    v6: socket_v6().ok().map(Arc::new),
                };
                if socket.v4.is_none() && socket.v6.is_none() {
                    return Err(SocketError::CannotBind);
                }
                Ok(socket)
            }
            _ => Ok(Self {
                v4: class
                    .has_v4()
                    .then(|| socket_v4().map(Arc::new))
                    .transpose()?,
                v6: class
                    .has_v6()
                    .then(|| socket_v6().map(Arc::new))
                    .transpose()?,
            }),
        }
    }

    /// Create sockets for a specific network interface.
    /// 
    /// For IPv4 addresses, creates a socket that joins the multicast group on that interface.
    /// For IPv6 addresses, you should also provide the interface index.
    /// 
    /// # Example
    /// ```no_run
    /// use std::net::{IpAddr, Ipv4Addr};
    /// use swarm_discovery::Sockets;
    /// 
    /// // Create sockets for a specific IPv4 interface
    /// let interface_ip = Ipv4Addr::new(192, 168, 1, 100);
    /// let sockets = Sockets::new_for_interface(IpAddr::V4(interface_ip), None).unwrap();
    /// ```
    pub fn new_for_interface(interface_addr: IpAddr, interface_index: Option<u32>) -> Result<Self, SocketError> {
        match interface_addr {
            IpAddr::V4(v4_addr) => {
                let v4_socket = socket_v4_on_interface(v4_addr)?;
                Ok(Self {
                    v4: Some(Arc::new(v4_socket)),
                    v6: None,
                })
            }
            IpAddr::V6(_) => {
                let v6_socket = socket_v6_on_interface(interface_index.unwrap_or(0))?;
                Ok(Self {
                    v4: None,
                    v6: Some(Arc::new(v6_socket)),
                })
            }
        }
    }

    pub fn v4(&self) -> Option<Arc<UdpSocket>> {
        self.v4.as_ref().map(Arc::clone)
    }

    pub fn v6(&self) -> Option<Arc<UdpSocket>> {
        self.v6.as_ref().map(Arc::clone)
    }

    pub async fn send_msg(&self, msg: &Message, mode: Mode) {
        let bytes = match msg.to_vec() {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("error serializing mDNS: {}", e);
                return;
            }
        };
        
        let (socket, addr) = match mode {
            Mode::V4 => {
                match &self.v4 {
                    Some(v4) => (v4, IpAddr::from(MDNS_IPV4)),
                    None => {
                        tracing::trace!("No IPv4 socket available, skipping IPv4 send");
                        return;
                    }
                }
            }
            Mode::V6 => {
                match &self.v6 {
                    Some(v6) => (v6, IpAddr::from(MDNS_IPV6)),
                    None => {
                        tracing::trace!("No IPv6 socket available, skipping IPv6 send");
                        return;
                    }
                }
            }
            Mode::Any => {
                if let Some(v4) = &self.v4 {
                    (v4, IpAddr::from(MDNS_IPV4))
                } else if let Some(v6) = &self.v6 {
                    (v6, IpAddr::from(MDNS_IPV6))
                } else {
                    tracing::warn!("No sockets available for sending");
                    return;
                }
            }
        };
        
        if let Err(e) = socket.send_to(&bytes, (addr, MDNS_PORT)).await {
            tracing::warn!("error sending mDNS: {}", e);
        } else {
            tracing::debug!(
                q = msg.queries().len(),
                an = msg.answers().len(),
                ad = msg.additionals().len(),
                "sent {} bytes",
                bytes.len()
            );
        }
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    V4,
    V6,
    Any,
}
