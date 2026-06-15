use crc32fast::Hasher;

/// Calculate IEEE 802.3 CRC32 of the given data.
fn calculate_crc32(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Generates a standard Serial Number based on the chip type and MAC address.
/// Example: PP-ESP32S3-2606-BATCH-0211BC
pub fn generate_serial_number(chip_type: &str, mac: &str) -> String {
    let clean_mac: String = mac.chars().filter(|c| c.is_alphanumeric()).collect();
    let clean_mac = clean_mac.to_uppercase();
    let last_6 = if clean_mac.len() >= 6 {
        &clean_mac[clean_mac.len() - 6..]
    } else {
        "000000"
    };
    let date_str = chrono::Local::now().format("%y%m").to_string();
    let clean_chip = chip_type.to_uppercase().replace("-", "");
    format!("PP-{}-{}-BATCH-{}", clean_chip, date_str, last_6)
}

/// Generates a standard Device Name based on the MAC address.
/// Example: PixelPad-0211BC
pub fn generate_device_name(mac: &str) -> String {
    let clean_mac: String = mac.chars().filter(|c| c.is_alphanumeric()).collect();
    let clean_mac = clean_mac.to_uppercase();
    let last_6 = if clean_mac.len() >= 6 {
        &clean_mac[clean_mac.len() - 6..]
    } else {
        "000000"
    };
    format!("PixelPad-{}", last_6)
}

/// Modifies the entry bitmap to mark slot `idx` as Written (10 binary).
fn set_entry_state(bitmap: &mut [u8; 32], idx: usize, state: u8) {
    let byte_idx = idx / 4;
    let bit_shift = (idx % 4) * 2;
    bitmap[byte_idx] &= !(0b11 << bit_shift);
    bitmap[byte_idx] |= (state & 0b11) << bit_shift;
}

/// Builds a 32-byte standard NVS entry.
/// Returns the entry bytes and the number of slots used.
fn make_nvs_entry(ns_idx: u8, el_type: u8, key: &str, data_val: &[u8], span: u8) -> [u8; 32] {
    let mut entry = [0xFFu8; 32];
    entry[0] = ns_idx;
    entry[1] = el_type;
    entry[2] = span;
    entry[3] = 0xFF; // chunk index (default/unused)

    // Key (bytes 8..24, null-padded)
    let key_bytes = key.as_bytes();
    let key_len = key_bytes.len().min(15);
    entry[8..8 + key_len].copy_from_slice(&key_bytes[..key_len]);
    if key_len < 16 {
        entry[8 + key_len] = 0; // null terminator
    }

    // Data (bytes 24..32)
    let data_len = data_val.len().min(8);
    entry[24..24 + data_len].copy_from_slice(&data_val[..data_len]);

    // Calculate Entry CRC32 over bytes 0..4, and 8..32 (total 28 bytes)
    let mut crc_buf = [0u8; 28];
    crc_buf[0..4].copy_from_slice(&entry[0..4]);
    crc_buf[4..20].copy_from_slice(&entry[8..24]);
    crc_buf[20..28].copy_from_slice(&entry[24..32]);

    let crc = calculate_crc32(&crc_buf);
    entry[4..8].copy_from_slice(&crc.to_le_bytes());

    entry
}

/// Generates a single, fully-compliant 4096-byte NVS page binary.
pub fn generate_nvs_page(serial_number: &str, device_name: &str) -> Vec<u8> {
    let mut page = vec![0xFFu8; 4096];

    // 1. Page Header (offsets 0..32)
    // Page State: Active (0xFFFFFFFE)
    page[0..4].copy_from_slice(&0xFFFFFFFEu32.to_le_bytes());
    // SeqNo: 0
    page[4..8].copy_from_slice(&0u32.to_le_bytes());
    // Version: 2
    page[8] = 0x02;

    // Calculate Page Header CRC32 (bytes 4..28)
    let header_crc = calculate_crc32(&page[4..28]);
    page[28..32].copy_from_slice(&header_crc.to_le_bytes());

    // 2. Setup Bitmaps (offsets 32..64)
    let mut bitmap = [0xFFu8; 32];

    // Slot tracking
    let mut current_slot = 0;

    // Slot 0: Namespace registration (span 1)
    set_entry_state(&mut bitmap, current_slot, 0b10);
    let ns_entry = make_nvs_entry(0, 0x01, "device", &[1, 0, 0, 0, 0, 0, 0, 0], 1);
    page[64..96].copy_from_slice(&ns_entry);
    current_slot += 1;

    // Slot 1: Serial Number Parent (span 1 + child_count)
    let sn_bytes = serial_number.as_bytes();
    let sn_len = sn_bytes.len();
    let sn_child_count = (sn_len + 31) / 32;
    let sn_total_span = 1 + sn_child_count;

    // Mark parent slot
    set_entry_state(&mut bitmap, current_slot, 0b10);
    // Mark child slots
    for i in 1..=sn_child_count {
        set_entry_state(&mut bitmap, current_slot + i, 0b10);
    }

    // Build Serial Number Parent Entry
    // Data value holds: string length (u16) + schema version (1 byte, 0x01) + 5 unused bytes (0xFF)
    let mut sn_data = [0xFFu8; 8];
    sn_data[0..2].copy_from_slice(&(sn_len as u16).to_le_bytes());
    sn_data[2] = 0x01; // schema version

    let sn_parent_entry = make_nvs_entry(1, 0x21, "serial_number", &sn_data, sn_total_span as u8);
    let sn_parent_offset = 64 + current_slot * 32;
    page[sn_parent_offset..sn_parent_offset + 32].copy_from_slice(&sn_parent_entry);

    // Build Serial Number Child Entries (just raw bytes copy)
    let sn_child_offset = 64 + (current_slot + 1) * 32;
    let mut sn_padded_bytes = vec![0u8; sn_child_count * 32];
    sn_padded_bytes[..sn_len].copy_from_slice(sn_bytes);
    page[sn_child_offset..sn_child_offset + sn_child_count * 32].copy_from_slice(&sn_padded_bytes);

    current_slot += sn_total_span;

    // Slot 2+c_sn: Device Name Parent (span 1 + child_count)
    let dn_bytes = device_name.as_bytes();
    let dn_len = dn_bytes.len();
    let dn_child_count = (dn_len + 31) / 32;
    let dn_total_span = 1 + dn_child_count;

    // Mark parent slot
    set_entry_state(&mut bitmap, current_slot, 0b10);
    // Mark child slots
    for i in 1..=dn_child_count {
        set_entry_state(&mut bitmap, current_slot + i, 0b10);
    }

    // Build Device Name Parent Entry
    let mut dn_data = [0xFFu8; 8];
    dn_data[0..2].copy_from_slice(&(dn_len as u16).to_le_bytes());
    dn_data[2] = 0x01;

    let dn_parent_entry = make_nvs_entry(1, 0x21, "device_name", &dn_data, dn_total_span as u8);
    let dn_parent_offset = 64 + current_slot * 32;
    page[dn_parent_offset..dn_parent_offset + 32].copy_from_slice(&dn_parent_entry);

    // Build Device Name Child Entries
    let dn_child_offset = 64 + (current_slot + 1) * 32;
    let mut dn_padded_bytes = vec![0u8; dn_child_count * 32];
    dn_padded_bytes[..dn_len].copy_from_slice(dn_bytes);
    page[dn_child_offset..dn_child_offset + dn_child_count * 32].copy_from_slice(&dn_padded_bytes);

    // Copy bitmap to page at offsets 32..64
    page[32..64].copy_from_slice(&bitmap);

    page
}
