use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    time::Instant,
};

use anyhow::bail;
use local_ip_address::local_ip;
use str0m::{
    change::SdpOffer,
    net::{Protocol, Receive},
    Candidate, Event, IceConnectionState, Input, Output, Rtc,
};

fn main() -> anyhow::Result<()> {
    // Instantiate a new Rtc instance.
    let mut rtc = Rtc::new();

    let local_ip = local_ip()?;
    dbg!(local_ip);
    let socket = UdpSocket::bind(format!("{local_ip}:0"))?;

    //  Add host candidate
    let addr = SocketAddr::new(local_ip, socket.local_addr()?.port());
    let candidate = Candidate::host(addr, "udp").unwrap();
    rtc.add_local_candidate(candidate);

    // Accept an incoming offer from the remote peer
    // and get the corresponding answer.
    println!("Paste offer here.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let offer: SdpOffer = serde_json::from_str(input.trim())?;
    let answer = rtc.sdp_api().accept_offer(offer).unwrap();
    println!("===ANSWER===\n{}", serde_json::to_string(&answer).unwrap());
    // run loop

    // Buffer for reading incoming UDP packets.
    let mut buf = vec![0; 2000];

    loop {
        // Poll output until we get a timeout. The timeout means we
        // are either awaiting UDP socket input or the timeout to happen.
        let timeout = match rtc.poll_output().unwrap() {
            // Stop polling when we get the timeout.
            Output::Timeout(v) => v,

            // Transmit this data to the remote peer. Typically via
            // a UDP socket. The destination IP comes from the ICE
            // agent. It might change during the session.
            Output::Transmit(v) => {
                socket.send_to(&v.contents, v.destination).unwrap();
                continue;
            }

            // Events are mainly incoming media data from the remote
            // peer, but also data channel data and statistics.
            Output::Event(v) => {
                // Abort if we disconnect.
                if v == Event::IceConnectionStateChange(IceConnectionState::Disconnected) {
                    bail!("Disconnected");
                }

                dbg!(v);

                continue;
            }
        };

        // Duration until timeout.
        let duration = timeout - Instant::now();

        // socket.set_read_timeout(Some(0)) is not ok
        if duration.is_zero() {
            // Drive time forwards in rtc straight away.
            rtc.handle_input(Input::Timeout(Instant::now())).unwrap();
            continue;
        }

        socket.set_read_timeout(Some(duration)).unwrap();

        // Scale up buffer to receive an entire UDP packet.
        buf.resize(2000, 0);

        // Try to receive. Because we have a timeout on the socket,
        // we will either receive a packet, or timeout.
        // This is where having an async loop shines. We can await multiple things to
        // happen such as outgoing media data, the timeout and incoming network traffic.
        // When using async there is no need to set timeout on the socket.
        let input = match socket.recv_from(&mut buf) {
            Ok((n, source)) => {
                // UDP data received.
                buf.truncate(n);
                Input::Receive(
                    Instant::now(),
                    Receive {
                        proto: Protocol::Udp,
                        source,
                        destination: socket.local_addr().unwrap(),
                        contents: buf.as_slice().try_into().unwrap(),
                    },
                )
            }

            Err(e) => match e.kind() {
                // Expected error for set_read_timeout().
                // One for windows, one for the rest.
                ErrorKind::WouldBlock | ErrorKind::TimedOut => Input::Timeout(Instant::now()),

                e => {
                    eprintln!("Error: {:?}", e);
                    bail!("Error: {}", e);
                }
            },
        };

        // Input is either a Timeout or Receive of data. Both drive the state forward.
        rtc.handle_input(input).unwrap();
    }
}
