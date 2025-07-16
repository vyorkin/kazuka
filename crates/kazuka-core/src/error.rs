use alloy::transports::{RpcError, TransportErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KazukaError {
    #[error("RPC error")]
    RpcError(#[from] RpcError<TransportErrorKind>),
}
