//! Modbus TCP master client (tokio-modbus).
//!
//! Connection lifetime is owned by the caller: every function here takes an
//! established [`Context`]. The engine keeps one polling connection and one
//! serialized write connection per device, so a burst of operator commands no
//! longer opens (and leaks) a TCP session per write.

use std::net::SocketAddr;
use std::time::Duration;

use thiserror::Error;
use tokio_modbus::client::{Context, Reader, Writer};
use tokio_modbus::prelude::*;

pub mod codec;

pub use tokio_modbus::client::Context as ModbusContext;

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

/// Resolve `host:port` through the OS resolver so DNS names and IPv6 literals
/// work, not only numeric IPv4 addresses.
pub async fn resolve(cfg: &ConnectionConfig) -> Result<SocketAddr, ModbusError> {
    let target = format!("{}:{}", cfg.host, cfg.port);
    if let Ok(direct) = target.parse::<SocketAddr>() {
        return Ok(direct);
    }
    let lookup_target = target.clone();
    match tokio::time::timeout(dur(cfg.timeout_ms), tokio::net::lookup_host(lookup_target)).await {
        Ok(Ok(mut addrs)) => addrs
            .next()
            .ok_or_else(|| ModbusError::Address(format!("{target} resolved to no address"))),
        Ok(Err(e)) => Err(ModbusError::Address(format!("{target}: {e}"))),
        Err(_) => Err(ModbusError::Timeout(cfg.timeout_ms)),
    }
}

fn dur(ms: u64) -> Duration {
    Duration::from_millis(ms.max(50))
}

pub async fn connect(cfg: &ConnectionConfig) -> Result<Context, ModbusError> {
    let socket = resolve(cfg).await?;
    let ctx = match tokio::time::timeout(dur(cfg.timeout_ms), tcp::connect(socket)).await {
        Ok(Ok(ctx)) => ctx,
        Ok(Err(e)) => return Err(ModbusError::Connection(e.to_string())),
        Err(_) => return Err(ModbusError::Timeout(cfg.timeout_ms)),
    };
    let mut ctx = ctx;
    ctx.set_slave(Slave(cfg.unit_id));
    Ok(ctx)
}

/// Close a context, ignoring errors from an already-dead socket.
pub async fn close(mut ctx: Context) {
    let _ = ctx.disconnect().await;
}

pub async fn test_connection(cfg: &ConnectionConfig) -> Result<(), ModbusError> {
    let ctx = connect(cfg).await?;
    close(ctx).await;
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

/// Write one holding register (FC06) and return the observed read-back.
pub async fn write_holding(
    ctx: &mut Context,
    address: u16,
    value: u16,
    timeout_ms: u64,
    verify_readback: bool,
) -> Result<u16, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.write_single_register(address, value)).await {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(exc))) => Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Write(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }?;
    let readback = read_holding(ctx, address, 1, timeout_ms)
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

/// Write a contiguous block of holding registers (FC16) and return the
/// observed read-back. Used for every data type wider than one register.
pub async fn write_holding_block(
    ctx: &mut Context,
    address: u16,
    values: &[u16],
    timeout_ms: u64,
    verify_readback: bool,
) -> Result<Vec<u16>, ModbusError> {
    if values.is_empty() {
        return Err(ModbusError::Write("empty register block".into()));
    }
    let quantity = u16::try_from(values.len())
        .map_err(|_| ModbusError::Write("register block too large".into()))?;
    match tokio::time::timeout(
        dur(timeout_ms),
        ctx.write_multiple_registers(address, values),
    )
    .await
    {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(exc))) => Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => Err(ModbusError::Write(io.to_string())),
        Err(_) => Err(ModbusError::Timeout(timeout_ms)),
    }?;
    let readback = read_holding(ctx, address, quantity, timeout_ms).await?;
    if readback.len() != values.len() {
        return Err(ModbusError::Read(format!(
            "short read-back at HR{address}: expected {} registers, got {}",
            values.len(),
            readback.len()
        )));
    }
    if verify_readback && readback != values {
        return Err(ModbusError::Write(format!(
            "read-back mismatch at HR{address}: requested {values:?}, got {readback:?}"
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
    ctx: &mut Context,
    address: u16,
    bit: u8,
    value: bool,
    timeout_ms: u64,
    verify_readback: bool,
) -> Result<u16, ModbusError> {
    let (and_mask, or_mask) = bit_write_masks(bit, value)?;
    match tokio::time::timeout(
        dur(timeout_ms),
        ctx.masked_write_register(address, and_mask, or_mask),
    )
    .await
    {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(timeout_ms)),
    }
    let readback = read_holding(ctx, address, 1, timeout_ms)
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
    ctx: &mut Context,
    address: u16,
    bit: u8,
    value: bool,
    timeout_ms: u64,
    verify_readback: bool,
) -> Result<u16, ModbusError> {
    let _ = bit_write_masks(bit, value)?;
    let current = read_holding(ctx, address, 1, timeout_ms)
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
        dur(timeout_ms),
        ctx.write_single_register(address, requested),
    )
    .await
    {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(timeout_ms)),
    }
    let readback = read_holding(ctx, address, 1, timeout_ms)
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
    ctx: &mut Context,
    address: u16,
    value: bool,
    timeout_ms: u64,
    verify_readback: bool,
) -> Result<bool, ModbusError> {
    match tokio::time::timeout(dur(timeout_ms), ctx.write_single_coil(address, value)).await {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(exc))) => return Err(ModbusError::Write(format!("exception: {exc:?}"))),
        Ok(Err(io)) => return Err(ModbusError::Write(io.to_string())),
        Err(_) => return Err(ModbusError::Timeout(timeout_ms)),
    }
    let readback = read_coils(ctx, address, 1, timeout_ms)
        .await?
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
    use super::*;

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

    #[tokio::test]
    async fn resolve_accepts_ip_literals_and_host_names() {
        let literal = ConnectionConfig {
            host: "127.0.0.1".into(),
            port: 5020,
            unit_id: 1,
            timeout_ms: 500,
        };
        let addr = resolve(&literal).await.expect("ip literal resolves");
        assert_eq!(addr.port(), 5020);

        let named = ConnectionConfig {
            host: "localhost".into(),
            ..literal
        };
        let addr = resolve(&named).await.expect("host name resolves");
        assert_eq!(addr.port(), 5020);
        assert!(addr.ip().is_loopback());
    }

    #[tokio::test]
    async fn resolve_reports_unknown_hosts_instead_of_silently_failing() {
        let cfg = ConnectionConfig {
            host: "proscada.invalid.example".into(),
            port: 502,
            unit_id: 1,
            timeout_ms: 500,
        };
        assert!(matches!(
            resolve(&cfg).await,
            Err(ModbusError::Address(_) | ModbusError::Timeout(_))
        ));
    }
}
