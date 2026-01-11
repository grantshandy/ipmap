#[cfg(target_os = "windows")]
pub const CHILD_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "\\..\\..\\target\\release\\ipmap-child.exe"
));

#[cfg(not(target_os = "windows"))]
pub const CHILD_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../target/release/ipmap-child"
));
