//! QUIC settings shared by the server and the player: transport, TLS and the UDP socket.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use quinn::crypto::rustls::QuicClientConfig;
use quinn::{
    ClientConfig, Endpoint, EndpointConfig, IdleTimeout, ServerConfig, SmolRuntime,
    TransportConfig, VarInt,
};
use socket2::{Domain, Protocol, Socket, Type};

use crate::trust::{Identity, Verifier};

/// How often a quiet connection proves it is alive.
const KEEP_ALIVE: Duration = Duration::from_secs(5);

/// How long a connection may stay silent before it counts as lost.
const IDLE_TIMEOUT: Duration = Duration::from_secs(15);

/// Round trip assumed before the first measure: a typical internet path.
const INITIAL_RTT: Duration = Duration::from_millis(100);

/// Bytes of datagrams kept waiting, each way: about two ticks of state for 16 players (15
/// others, three records each, about 60 bytes a record). Older datagrams are dropped first,
/// so a stale pose is never queued behind a fresh one.
const DATAGRAM_BUFFER: usize = 8 * 1024;

/// Bytes of the UDP socket's own buffers, raised so a burst of 16 players is not dropped.
const SOCKET_BUFFER: usize = 1 << 20;

/// Returns the transport settings. `streams` is how many streams the other side may open.
fn transport(streams: u32) -> Arc<TransportConfig> {
    let mut config = TransportConfig::default();
    config
        .keep_alive_interval(Some(KEEP_ALIVE))
        .max_idle_timeout(IdleTimeout::try_from(IDLE_TIMEOUT).ok())
        .initial_rtt(INITIAL_RTT)
        .datagram_receive_buffer_size(Some(DATAGRAM_BUFFER))
        .datagram_send_buffer_size(DATAGRAM_BUFFER)
        .max_concurrent_bidi_streams(VarInt::from_u32(streams))
        .max_concurrent_uni_streams(VarInt::from_u32(0));
    Arc::new(config)
}

/// Returns the server's QUIC settings: its certificate, and one stream per player.
pub(crate) fn server(identity: &Identity) -> io::Result<ServerConfig> {
    let mut config = ServerConfig::with_single_cert(vec![identity.cert().clone()], identity.key())
        .map_err(io::Error::other)?;
    config.transport_config(transport(1));
    Ok(config)
}

/// Returns a player's QUIC settings, which trust the server through `verifier`.
pub(crate) fn client(verifier: Arc<Verifier>) -> io::Result<ClientConfig> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(io::Error::other)?
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();
    let quic = QuicClientConfig::try_from(tls).map_err(io::Error::other)?;
    let mut config = ClientConfig::new(Arc::new(quic));
    config.transport_config(transport(0));
    Ok(config)
}

/// Opens a QUIC endpoint on `bind`, with larger socket buffers. With a server configuration
/// it takes connections; without, it only makes them.
pub(crate) fn endpoint(bind: SocketAddr, server: Option<ServerConfig>) -> io::Result<Endpoint> {
    let socket = socket(bind)?;
    Endpoint::new(
        EndpointConfig::default(),
        server,
        socket,
        Arc::new(SmolRuntime),
    )
}

/// Opens a UDP socket on `bind` with raised buffers. A system that refuses the size keeps its
/// own: the socket still works.
fn socket(bind: SocketAddr) -> io::Result<UdpSocket> {
    let socket = Socket::new(Domain::for_address(bind), Type::DGRAM, Some(Protocol::UDP))?;
    let _ = socket.set_recv_buffer_size(SOCKET_BUFFER);
    let _ = socket.set_send_buffer_size(SOCKET_BUFFER);
    socket.bind(&bind.into())?;
    Ok(socket.into())
}
