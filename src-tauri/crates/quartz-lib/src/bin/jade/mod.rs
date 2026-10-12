pub mod hash_manager;
pub mod reader;
pub mod text_reader;
pub mod text_writer;
pub mod types;
pub mod unhasher;
pub mod writer;

/// Convert binary .bin data to ritobin text format.
/// Pipeline: binary → read → unhash → text
pub fn convert_bin_to_text(data: &[u8]) -> Result<String, String> {
    // Check if already text
    if let Ok(text) = std::str::from_utf8(data) {
        let trimmed = text.trim_start();
        if trimmed.starts_with("#PROP") || trimmed.starts_with("#PTCH") {
            return Ok(text.to_string());
        }
    }

    let bin = super::ritoshark_bridge::read_bin(data).map_err(|e| e.to_string())?;
    super::ritoshark_bridge::tree_to_text_cached(&bin).map_err(|e| e.to_string())
}

/// Convert ritobin text to binary .bin format.
/// Pipeline: text → parse → write binary
pub fn convert_text_to_bin(text: &str) -> Result<Vec<u8>, String> {
    let bin = super::ritoshark_bridge::text_to_tree(text).map_err(|e| e.to_string())?;
    super::ritoshark_bridge::write_bin(&bin).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn public_conversion_preserves_wide_hashes() {
        use crate::bin::ritoshark_bridge;
        use ritoshark::bin::{Bin, BinEntry, BinValue};
        let mut bin = Bin::new();
        bin.entries.push(BinEntry {
            path_hash: 1,
            class_hash: 2,
            fields: [(3, BinValue::Hash64(7)), (4, BinValue::Hash64(u64::MAX))]
                .into_iter().collect(),
        });
        let bytes = ritoshark_bridge::write_bin(&bin).unwrap();
        let text = super::convert_bin_to_text(&bytes).unwrap();
        assert!(text.contains("0x0000000000000007"));
        assert!(text.contains("0xffffffffffffffff"));
        assert_eq!(super::convert_text_to_bin(&text).unwrap(), bytes);
    }
}
