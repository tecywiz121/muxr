use crate::error::*;

use serde::de::DeserializeOwned;
use serde::Serialize;

use std::io::Read;

pub fn serialize<T: Serialize>(obj: &T) -> Result<Vec<u8>> {
    const USZ_LEN: usize = std::mem::size_of::<usize>();

    let mut buffer = vec![0u8; USZ_LEN];

    bincode::serialize_into(&mut buffer, obj)?;

    let len = buffer.len().to_ne_bytes();
    buffer[0..USZ_LEN].copy_from_slice(&len);

    Ok(buffer)
}

pub fn deserialize_from<T: DeserializeOwned>(reader: &mut dyn Read) -> Result<T> {
    let mut sz_buf = [0u8; std::mem::size_of::<usize>()];
    reader.read_exact(&mut sz_buf)?;

    let sz = usize::from_ne_bytes(sz_buf) - sz_buf.len();
    let mut msg_buf = vec![0u8; sz];

    reader.read_exact(&mut msg_buf)?;

    Ok(bincode::deserialize(&msg_buf)?)
}
