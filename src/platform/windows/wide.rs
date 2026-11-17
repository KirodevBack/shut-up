pub fn null_terminated(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

pub fn copy_into(dest: &mut [u16], text: &str) {
    let room = dest.len().saturating_sub(1);
    for (slot, unit) in dest[..room].iter_mut().zip(text.encode_utf16()) {
        *slot = unit;
    }
}

pub fn to_ascii<'a>(source: &[u16], buffer: &'a mut [u8]) -> Option<&'a str> {
    let len = source.iter().position(|&c| c == 0).unwrap_or(source.len());
    let source = &source[..len];
    if source.len() > buffer.len() || source.iter().any(|&c| c > 0x7f) {
        return None;
    }
    for (dst, &src) in buffer.iter_mut().zip(source) {
        *dst = src as u8;
    }
    std::str::from_utf8(&buffer[..len]).ok()
}
