use std::io::IoSliceMut;
use std::os::fd::AsRawFd;
use nix::libc::{IPTOS_ECN_CE, IPTOS_ECN_ECT0, IPTOS_ECN_ECT1};
use nix::sys::socket::sockopt::IpRecvTos;
use slog::{warn, Drain};
use slog::{error, info};
use std::net::{IpAddr, Ipv4Addr, UdpSocket, SocketAddr};
use nix::sys::socket::{sendto, recvmsg, ControlMessageOwned, MsgFlags, SetSockOpt, SockaddrIn};
use nix::cmsg_space;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value_t=IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    ip_addr: IpAddr,

    #[arg(default_value_t=5000)]
    port: u16,

}

fn main() -> Result<(), std::io::Error> {

    let args = Args::parse();

    let decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let drain = slog_term::FullFormat::new(decorator)
        .build()
        .filter_level(slog::Level::Info)
        .fuse();
    let logger = slog::Logger::root(drain, slog::o!("version" => "0.5"));

    let bind_socket_addr = SocketAddr::from((args.ip_addr, args.port));
    let bind_result = UdpSocket::bind(bind_socket_addr);

    if let Err(bind_err) = bind_result {
        error!(logger, "There was an error creating the server socket: {}", bind_err);
        return Err(bind_err);
    }
    let socket = bind_result.unwrap();

    let recv_tos_value = true;
    if let Err(set_recv_tos_err) = IpRecvTos.set(&socket, &recv_tos_value) {
        error!(logger, "There was an error configuring the server socket: {}", set_recv_tos_err);
        return Err(std::io::ErrorKind::ConnectionRefused.into());
    }

    loop {

        let mut recv_buffer = [0u8; 4];
        let recv_buffer_iov = IoSliceMut::new(&mut recv_buffer);

        let mut cmsg = cmsg_space!(u8);
        let mut iovs = [recv_buffer_iov];

        let recv_result = recvmsg::<SockaddrIn>(socket.as_raw_fd(), &mut iovs, Some(&mut cmsg), MsgFlags::empty());
        let mut response_buffer: Option<[u8;4]> = None;
        let mut client_address: Option<SockaddrIn> = None;
        if let Ok(recv_msg) = recv_result {
            for cmsg in recv_msg.cmsgs().unwrap() {
                if let ControlMessageOwned::Ipv4Tos(ecn_value) = cmsg {
                    response_buffer = Some(match ecn_value {
                        IPTOS_ECN_CE => [b'C', b'E', 0u8, 0u8],
                        IPTOS_ECN_ECT0 => [b'E', b'C', b'T', b'0'],
                        IPTOS_ECN_ECT1 => [b'E', b'C', b'T', b'1'],
                        _ => [b'N', b'O', b'N',b'E']
                    });
                    info!(logger, "Got a TOS: {}", ecn_value);
                } else {
                    warn!(logger, "Skipping non-TOS-related control message: {:?}", cmsg)
                }
            }
            client_address = recv_msg.address;
        } else {
            error!(logger, "There was an error on recv msg: {:?}", recv_result)
        }

        if client_address.is_none() {
            warn!(logger, "Did not get a client address; not responding to probe.");
            continue;
        }
        let client_address = client_address.unwrap();

        let send_result = response_buffer.ok_or(std::io::Error::other("value")).and_then(|buffer| {
            sendto(socket.as_raw_fd(), &buffer, &client_address, MsgFlags::empty()).map_err(|e| std::io::Error::other(e.to_string()))
        });
        if let Err(send_err) = send_result {
            error!(logger, "There was an error sending the response: {}", send_err);
        }
    }
    Ok(())
}
