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
    match tokio::time::timeout(
        dur(timeout_ms),
        ctx.read_holding_registers(address, quantity),
    )
    .await
    {
        Ok(Ok(Ok(values))) => Ok(values),
        Ok(Ok(Err(exc))) => Err(ModbusError::Read(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Read(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }
}

pub async fn read_input(
    ctx: &mut Context,
    address: u16,
    quantity: u16,
    timeout_ms: u64,
) -> Result<Vec<u16>, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.read_input_registers(address, quantity)).await {
        Ok(Ok(Ok(values))) => Ok(values),
        Ok(Ok(Err(exc))) => Err(ModbusError::Read(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Read(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }
}

pub async fn read_coils(
    ctx: &mut Context,
    address: u16,
    quantity: u16,
    timeout_ms: u64,
) -> Result<Vec<bool>, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.read_coils(address, quantity)).await {
        Ok(Ok(Ok(values))) => Ok(values),
        Ok(Ok(Err(exc))) => Err(ModbusError::Read(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Read(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }
}

pub async fn read_discrete(
    ctx: &mut Context,
    address: u16,
    quantity: u16,
    timeout_ms: u64,
) -> Result<Vec<bool>, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.read_discrete_inputs(address, quantity)).await {
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
    verify_readback: bool,
) -> Result<u16, ModbusError> {
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
    }?;
    let readback = read_holding(&mut ctx, address, 1, cfg.timeout_ms)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| ModbusError::Read("empty read-back response".into()))?;
    if verify_readback && readback != value {
        return Err(ModbusError::Write(format!(
            "read-back mismatch at HR{address}: requested {value}, got {readback}"
        )));
    }
    Ok(readback)
}

pub fn bit_write_masks(bit: u8, value: bool) -> Result<(u16, u16), ModbusError> {
    if bit > 15 {
        return Err(ModbusError::Address(format!(
            "holding-register bit must be 0..15, got {bit}"
        )));
    }
    let mask = 1_u16 << bit;
    Ok((!mask, if value { mask } else { 0 }))
}

pub async fn write_holding_bit_masked(
    cfg: &ConnectionConfig,
    address: u16,
    bit: u8,
    value: bool,
    verify_readback: bool,
) -> Result<u16, ModbusError> {
    let (and_mask, or_mask) = bit_write_masks(bit, value)?;
    let mut ctx = connect(cfg).await?;
    match tokio::time::timeout(
        dur(cfg.timeout_ms),
        ctx.masked_write_register(address, and_mask, or_mask),
    )
    .await
    {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    }
    let readback = read_holding(&mut ctx, address, 1, cfg.timeout_ms)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| ModbusError::Read("empty bit read-back response".into()))?;
    let actual = ((readback >> bit) & 1) == 1;
    if verify_readback && actual != value {
        return Err(ModbusError::Write(format!(
            "bit read-back mismatch at HR{address}.{bit}: requested {value}, got {actual}"
        )));
    }
    Ok(readback)
}

pub async fn write_holding_bit_rmw(
    cfg: &ConnectionConfig,
    address: u16,
    bit: u8,
    value: bool,
    verify_readback: bool,
) -> Result<u16, ModbusError> {
    let _ = bit_write_masks(bit, value)?;
    let mut ctx = connect(cfg).await?;
    let current = read_holding(&mut ctx, address, 1, cfg.timeout_ms)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| ModbusError::Read("empty RMW read response".into()))?;
    let mask = 1_u16 << bit;
    let requested = if value {
        current | mask
    } else {
        current & !mask
    };
    match tokio::time::timeout(
        dur(cfg.timeout_ms),
        ctx.write_single_register(address, requested),
    )
    .await
    {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    }
    let readback = read_holding(&mut ctx, address, 1, cfg.timeout_ms)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| ModbusError::Read("empty RMW read-back response".into()))?;
    let actual = ((readback >> bit) & 1) == 1;
    if verify_readback && actual != value {
        return Err(ModbusError::Write(format!(
            "RMW bit mismatch at HR{address}.{bit}: requested {value}, got {actual}"
        )));
    }
    Ok(readback)
}

pub async fn write_coil(
    cfg: &ConnectionConfig,
    address: u16,
    value: bool,
    verify_readback: bool,
) -> Result<bool, ModbusError> {
    let mut ctx = connect(cfg).await?;
    match tokio::time::timeout(dur(cfg.timeout_ms), ctx.write_single_coil(address, value)).await {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    }
    let values = match tokio::time::timeout(dur(cfg.timeout_ms), ctx.read_coils(address, 1)).await {
        Ok(Ok(Ok(values))) => values,
        Ok(Ok(Err(exc))) => return Err(ModbusError::Read(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Read(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    };
    let readback = values
        .into_iter()
        .next()
        .ok_or_else(|| ModbusError::Read("empty coil read-back response".into()))?;
    if verify_readback && readback != value {
        return Err(ModbusError::Write(format!(
            "coil read-back mismatch at C{address}: requested {value}, got {readback}"
        )));
    }
    Ok(readback)
}

#[cfg(test)]
mod tests {
    use super::bit_write_masks;

    #[test]
    fn mask_write_sets_only_requested_bit() {
        let current = 0xA55A_u16;
        let (and_mask, or_mask) = bit_write_masks(2, true).expect("valid bit");
        let result = (current & and_mask) | (or_mask & !and_mask);
        assert_eq!(and_mask, 0xFFFB);
        assert_eq!(or_mask, 0x0004);
        assert_eq!(result, 0xA55E);
    }

    #[test]
    fn mask_write_resets_only_requested_bit() {
        let current = 0xA55E_u16;
        let (and_mask, or_mask) = bit_write_masks(2, false).expect("valid bit");
        let result = (current & and_mask) | (or_mask & !and_mask);
        assert_eq!(and_mask, 0xFFFB);
        assert_eq!(or_mask, 0x0000);
        assert_eq!(result, 0xA55A);
    }

    #[test]
    fn mask_write_rejects_non_physical_bit() {
        assert!(bit_write_masks(16, true).is_err());
    }
}
