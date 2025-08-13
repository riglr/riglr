pub fn discriminator_matches(data: &str, discriminator: &str) -> bool {
    if !data.starts_with("0x") || !discriminator.starts_with("0x") {
        return false;
    }
    let dlen = discriminator.len();
    data.len() >= dlen && &data[..dlen] == discriminator
}
