use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket as StdUdpSocket},
};

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;

pub(super) fn bind(port: u16) -> io::Result<UdpSocket> {
    let socket = match bind_dual_stack(port) {
        Ok(socket) => socket,
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => return Err(error),
        Err(_) => StdUdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?,
    };
    socket.set_nonblocking(true)?;
    UdpSocket::from_std(socket)
}

fn bind_dual_stack(port: u16) -> io::Result<StdUdpSocket> {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_only_v6(false)?;
    socket.bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)).into())?;
    Ok(socket.into())
}

pub(super) async fn wait(socket: UdpSocket) -> io::Result<()> {
    // The extra byte keeps an oversized datagram from passing as a valid 13-byte query.
    let mut packet = [0; 14];
    loop {
        let (size, _) = socket.recv_from(&mut packet).await?;
        if packet.get(..size).is_some_and(is_poll) {
            return Ok(());
        }
    }
}

const fn is_poll(packet: &[u8]) -> bool {
    // Magic, poll type, protocol version, eight-byte cookie, terminator.
    matches!(packet, [0xd5, 0xf6, 0, 1, _, _, _, _, _, _, _, _, 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_a_complete_version_one_poll() {
        let mut poll = vec![0xd5, 0xf6, 0, 1, 42, 0, 0, 0, 0, 0, 0, 0, 1];
        assert!(is_poll(&poll));
        assert!(!is_poll(&poll[..12]));
        poll.push(1);
        assert!(!is_poll(&poll));
        poll.pop();
        for index in [0, 1, 2, 3, 12] {
            let mut invalid = poll.clone();
            invalid[index] ^= 1;
            assert!(!is_poll(&invalid));
        }
    }
}
