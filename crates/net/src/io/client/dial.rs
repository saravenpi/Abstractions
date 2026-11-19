//! Reaching a server: the address, the endpoint, the handshake and its verdict.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use quinn::{Connection, ConnectionError, Endpoint};

use super::CloseReason;
use crate::io::addr::resolve;
use crate::io::{close, config};
use crate::trust::{SERVER_NAME, Verifier};

/// Resolves `address`, which blocks, then makes the handshake checked by `verifier`.
pub(super) async fn dial(
    address: &str,
    verifier: &Arc<Verifier>,
) -> Result<(Endpoint, Connection), CloseReason> {
    let target = resolve(address).map_err(CloseReason::Address)?;
    let any = match target {
        SocketAddr::V4(_) => SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
        SocketAddr::V6(_) => SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0)),
    };
    let endpoint = config::endpoint(any, None).map_err(|_| CloseReason::Unreachable)?;
    let settings = config::client(verifier.clone()).map_err(|_| CloseReason::Lost)?;
    let connecting = endpoint
        .connect_with(settings, target, SERVER_NAME)
        .map_err(|_| CloseReason::Unreachable)?;
    match connecting.await {
        Ok(conn) => Ok((endpoint, conn)),
        Err(error) => Err(match verifier.outcome() {
            Some(Err(trust)) => CloseReason::Trust(trust),
            _ if error == ConnectionError::TimedOut => CloseReason::Unreachable,
            _ => ended(error),
        }),
    }
}

/// Returns why a connection that was up has ended.
pub(super) fn ended(error: ConnectionError) -> CloseReason {
    match error {
        ConnectionError::ApplicationClosed(closed) => {
            close::reason(closed.error_code).map_or(CloseReason::Lost, CloseReason::Refused)
        }
        ConnectionError::TimedOut => CloseReason::Timeout,
        _ => CloseReason::Lost,
    }
}
