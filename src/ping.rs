use rand::random;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, IpAddr};

use errors::{Error, ErrorKind};
use packet::{EchoReply, EchoRequest, IpV4Packet, IcmpV4, IcmpV6, ICMP_HEADER_SIZE};

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];

pub fn ping(addr: IpAddr) -> Result<(), Error> {
    let dest = SocketAddr::new(addr, 0);
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let token: Token = random();
    let ident = random();

    let request = EchoRequest {
        ident: ident,
        seq_cnt: 1,
        payload: &token,
    };

    let socket = if dest.is_ipv4() {
        try!(Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4())))
    } else {
        try!(Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6())))
    };

    if dest.is_ipv4() {
        if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
            return Err(ErrorKind::InternalError.into());
        }
    } else {
        if request.encode::<IcmpV6>(&mut buffer[..]).is_err() {
            return Err(ErrorKind::InternalError.into());
        }
    }

    try!(socket.send_to(&mut buffer, &dest.into()));

    let mut buffer: [u8; 2048] = [0; 2048];
    try!(socket.recv_from(&mut buffer));

    let reply = if dest.is_ipv4() {
        let ipv4_packet = match IpV4Packet::decode(&buffer) {
            Ok(packet) => packet,
            Err(_) => return Err(ErrorKind::InternalError.into()),
        };
        match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
            Ok(reply) => reply,
            Err(_) => return Err(ErrorKind::InternalError.into()),
        }
    } else {
        match EchoReply::decode::<IcmpV6>(&buffer) {
            Ok(reply) => reply,
            Err(_) => return Err(ErrorKind::InternalError.into()),
        }
    };
    assert!(reply.ident == request.ident);
    assert!(reply.seq_cnt == request.seq_cnt);
    return Ok(());
}