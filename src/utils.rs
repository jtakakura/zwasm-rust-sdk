use crate::error;

pub(crate) fn to_u32_len(len: usize) -> Result<u32, error::ZwasmError> {
    u32::try_from(len).map_err(|_| error::ZwasmError(format!("length exceeds u32::MAX: {len}")))
}
