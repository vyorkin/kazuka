use alloy::transports::{RpcError, TransportErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KazukaError {
    #[error("RPC error")]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error("CSV error: `{0}`")]
    CsvError(String),
}
