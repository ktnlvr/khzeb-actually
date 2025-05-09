pub fn read_cstring(memory: &[u8], ptr: u32) -> String {
    let mut end = ptr as usize;
    while memory[end] != 0 {
        end += 1;
    }
    let bytes = &memory[ptr as usize..end];
    String::from_utf8_lossy(bytes).to_string()
}

pub fn read_pointer(memory: &[u8], ptr: u32) -> u32 {
    u32::from_le_bytes([
        memory[ptr as usize],
        memory[ptr as usize + 1],
        memory[ptr as usize + 2],
        memory[ptr as usize + 3],
    ])
}
