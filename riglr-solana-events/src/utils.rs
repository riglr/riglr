/// Checks if a hex data string starts with a given hex discriminator
///
/// Both `data` and `discriminator` must be valid hex strings starting with "0x".
/// Returns `true` if the data string begins with the discriminator string.
///
/// # Arguments
///
/// * `data` - The hex data string to check
/// * `discriminator` - The hex discriminator to match against
///
/// # Returns
///
/// `true` if the data starts with the discriminator, `false` otherwise
pub fn discriminator_matches(data: &str, discriminator: &str) -> bool {
    if !data.starts_with("0x") || !discriminator.starts_with("0x") {
        return false;
    }
    let dlen = discriminator.len();
    data.len() >= dlen && &data[..dlen] == discriminator
}
