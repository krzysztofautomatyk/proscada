//! Modbus TCP master client (tokio-modbus).

use std::net::SocketAddr;
use std::time::Duration;

use thiserror::Error;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub unit_id: u8,
    pub timeout_ms: u64,
}

#[derive(Error, Debug)]
pub enum ModbusError {
    #[error("Invalid address: {0}")]
    Address(String),
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Read failed: {0}")]
    Read(String),
    #[error("Write failed: {0}")]
    Write(String),
    #[error("Timeout after {0} ms")]
    Timeout(u64),
}

fn addr(cfg: &ConnectionConfig) -> Result<SocketAddr, ModbusError> {
    format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .map_err(|e: std::net::AddrParseError| ModbusError::Address(e.to_string()))
}

fn dur(ms: u64) -> Duration {
    Duration::from_millis(ms.max(50))
}

pub async fn connect(cfg: &ConnectionConfig) -> Result<Context, ModbusError> {
    let socket = addr(cfg)?;
    let ctx = match tokio::time::timeout(dur(cfg.timeout_ms), tcp::connect(socket)).await {
        Ok(Ok(ctx)) => ctx,
        Ok(Err(e)) => return Err(ModbusError::Connection(e.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    };
    let mut ctx = ctx;
    ctx.set_slave(Slave(cfg.unit_id));
    Ok(ctx)
}

pub async fn test_connection(cfg: &ConnectionConfig) -> Result<(), ModbusError> {
    let _ = connect(cfg).await?;
    Ok(())
}

pub async fn read_holding(
    ctx: &mut Context,
    address: u16,
    quantity: u16,
    timeout_ms: u64,
) -> Result<Vec<u16>, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.read_holding_registers(address, quantity)).await
    {
        Ok(Ok(Ok(values))) => Ok(values),
        Ok(Ok(Err(exc))) => Err(ModbusError::Read(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Read(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }
}

pub async fn write_holding(
    cfg: &ConnectionConfig,
    address: u16,
    value: u16,
) -> Result<(), ModbusError> {
    let mut ctx = connect(cfg).await?;
    match tokio::time::timeout(
        dur(cfg.timeout_ms),
        ctx.write_single_register(address, value),
    )
    .await
    {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(exc))) => Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Write(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(cfg.timeout_ms)),
    }
}
