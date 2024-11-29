use std::sync::LazyLock;
pub static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

pub static NUM_IN_CHANNELS: u8 = 2;
pub static NUM_OUT_CHANNELS: u8 = 2;
