//! Binary Feature Extractor
//!
//! Extracts features from PE/ELF/Mach-O binaries using goblin.
//! This module provides the foundation for all binary-based detectors.
//!
//! Note: This is a simplified implementation focusing on string extraction
//! and basic format detection. Full import/export table parsing requires
//! more complex goblin API usage.

#[cfg(feature = "binary")]
use goblin::{elf::Elf, mach::MachO, pe::PE};

/// Binary format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// Windows PE (.dll, .exe)
    PE,
    /// Linux ELF (.so)
    ELF,
    /// macOS Mach-O (.dylib)
    MachO,
    /// Unknown or unsupported format
    Unknown,
}

/// An imported function entry
#[derive(Debug, Clone)]
pub struct ImportEntry {
    /// Name of the imported function
    pub name: String,
    /// Name of the DLL/library (if available)
    pub library: Option<String>,
}

/// An exported function entry
#[derive(Debug, Clone)]
pub struct ExportEntry {
    /// Name of the exported function
    pub name: String,
    /// RVA of the export (if available)
    pub rva: Option<u32>,
}

/// An extracted string from the binary
#[derive(Debug, Clone)]
pub struct ExtractedString {
    /// String content (ASCII/UTF-8 printable)
    pub content: String,
    /// Offset in the binary
    pub offset: usize,
    /// Length of the string
    pub length: usize,
}

/// Section information
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// Section name
    pub name: String,
    /// Virtual size of the section
    pub virtual_size: u64,
    /// Entropy of the section (0.0-8.0)
    pub entropy: f64,
    /// Section flags/characteristics
    pub flags: u32,
}

/// Debug information extracted from the binary
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// PDB path (for PE files)
    pub pdb_path: Option<String>,
    /// Cargo registry paths (for Rust binaries)
    pub cargo_paths: Vec<String>,
    /// Build timestamp (if available)
    pub build_timestamp: Option<u64>,
}

/// Binary features extracted from a .node file
#[derive(Debug, Clone)]
pub struct BinaryFeatures {
    /// Detected binary format
    pub format: BinaryFormat,
    /// Imported functions
    pub imports: Vec<ImportEntry>,
    /// Exported functions
    pub exports: Vec<ExportEntry>,
    /// Extracted strings (min 4 chars)
    pub strings: Vec<ExtractedString>,
    /// Section information with entropy
    pub sections: Vec<SectionInfo>,
    /// Debug information
    pub debug_info: Option<DebugInfo>,
    /// Raw binary data (for additional analysis)
    pub data: Vec<u8>,
}

impl BinaryFeatures {
    /// Create empty binary features
    pub fn empty() -> Self {
        Self {
            format: BinaryFormat::Unknown,
            imports: Vec::new(),
            exports: Vec::new(),
            strings: Vec::new(),
            sections: Vec::new(),
            debug_info: None,
            data: Vec::new(),
        }
    }
}

/// Detect binary format from magic bytes
#[cfg(feature = "binary")]
fn detect_format(data: &[u8]) -> BinaryFormat {
    if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {  // MZ
        BinaryFormat::PE
    } else if data.len() >= 4 && data[0] == 0x7F && &data[1..4] == b"ELF" {
        BinaryFormat::ELF
    } else if data.len() >= 4 {
        // Mach-O magic numbers (little/big endian 32/64-bit)
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic == 0xFEEDFACE || magic == 0xFEEDFACF || magic == 0xCAFEBABE {
            BinaryFormat::MachO
        } else {
            BinaryFormat::Unknown
        }
    } else {
        BinaryFormat::Unknown
    }
}

/// Extract binary features from raw bytes
#[cfg(feature = "binary")]
pub fn extract_features(data: &[u8]) -> Result<BinaryFeatures, String> {
    let format = detect_format(data);

    let mut features = BinaryFeatures::empty();
    features.data = data.to_vec();
    features.format = format;

    // Extract strings first (works for all formats)
    features.strings = extract_strings(data);

    // Parse based on format
    match format {
        BinaryFormat::PE => {
            if let Ok(pe) = PE::parse(data) {
                features.imports = extract_pe_imports(&pe);
                features.exports = extract_pe_exports(&pe);
                features.sections = extract_pe_sections(&pe, data);
                features.debug_info = extract_pe_debug_info(&pe, data);
            }
        }
        BinaryFormat::ELF => {
            if let Ok(elf) = Elf::parse(data) {
                features.imports = extract_elf_imports(&elf);
                features.exports = extract_elf_exports(&elf);
                features.sections = extract_elf_sections(&elf, data);
            }
        }
        BinaryFormat::MachO => {
            if let Ok(macho) = MachO::parse(data, 0) {
                features.imports = extract_macho_imports(&macho);
                features.exports = extract_macho_exports(&macho);
                features.sections = extract_macho_sections(&macho, data);
            }
        }
        BinaryFormat::Unknown => {
            return Err("Unknown binary format".to_string());
        }
    }

    Ok(features)
}

/// Extract printable ASCII/UTF-8 strings (min 4 chars)
fn extract_strings(data: &[u8]) -> Vec<ExtractedString> {
    let mut strings = Vec::new();
    let mut current_string = Vec::new();
    let mut start_offset = 0;

    for (i, &byte) in data.iter().enumerate() {
        if byte >= 0x20 && byte < 0x7F {
            if current_string.is_empty() {
                start_offset = i;
            }
            current_string.push(byte);
        } else {
            if current_string.len() >= 4 {
                if let Ok(s) = std::str::from_utf8(&current_string) {
                    strings.push(ExtractedString {
                        content: s.to_string(),
                        offset: start_offset,
                        length: current_string.len(),
                    });
                }
            }
            current_string.clear();
        }
    }

    // Handle last string
    if current_string.len() >= 4 {
        if let Ok(s) = std::str::from_utf8(&current_string) {
            strings.push(ExtractedString {
                content: s.to_string(),
                offset: start_offset,
                length: current_string.len(),
            });
        }
    }

    strings
}

/// Extract PE sections with entropy
#[cfg(feature = "binary")]
fn extract_pe_sections(pe: &PE, data: &[u8]) -> Vec<SectionInfo> {
    pe.sections
        .iter()
        .map(|section| {
            let name = section.name().unwrap_or("unknown").to_string();
            let size = section.size_of_raw_data as u64;
            let offset = section.pointer_to_raw_data as usize;
            let flags = section.characteristics;

            let entropy = if size > 0 && offset + size as usize <= data.len() {
                calculate_entropy(&data[offset..offset + size as usize])
            } else {
                0.0
            };

            SectionInfo {
                name,
                virtual_size: size,
                entropy,
                flags,
            }
        })
        .collect()
}

/// Extract ELF sections with entropy
#[cfg(feature = "binary")]
fn extract_elf_sections(elf: &Elf, data: &[u8]) -> Vec<SectionInfo> {
    elf.section_headers
        .iter()
        .map(|shdr| {
            let name = elf.shdr_strtab.get_at(shdr.sh_name).unwrap_or("unknown").to_string();
            let size = shdr.sh_size;
            let offset = shdr.sh_offset as usize;
            let flags = shdr.sh_flags as u32;

            let entropy = if size > 0 && offset + size as usize <= data.len() {
                calculate_entropy(&data[offset..offset + size as usize])
            } else {
                0.0
            };

            SectionInfo {
                name,
                virtual_size: size,
                entropy,
                flags,
            }
        })
        .collect()
}

/// Extract Mach-O sections with entropy
#[cfg(feature = "binary")]
fn extract_macho_sections(macho: &MachO, data: &[u8]) -> Vec<SectionInfo> {
    macho.segments
        .iter()
        .filter_map(|segment| {
            segment.name().ok().map(|name| {
                let size = segment.vmsize;
                let offset = segment.fileoff as usize;
                let flags = segment.flags;

                let entropy = if size > 0 && offset + size as usize <= data.len() {
                    calculate_entropy(&data[offset..offset + size as usize])
                } else {
                    0.0
                };

                SectionInfo {
                    name: name.to_string(),
                    virtual_size: size,
                    entropy,
                    flags,
                }
            })
        })
        .collect()
}

/// Calculate Shannon entropy of a byte slice (0.0-8.0)
fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0u32; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Extract imports from PE file
#[cfg(feature = "binary")]
fn extract_pe_imports(pe: &PE) -> Vec<ImportEntry> {
    pe.imports
        .iter()
        .map(|import| ImportEntry {
            name: import.name.to_string(),
            library: Some(import.dll.to_string()),
        })
        .collect()
}

/// Extract exports from PE file
#[cfg(feature = "binary")]
fn extract_pe_exports(pe: &PE) -> Vec<ExportEntry> {
    pe.exports
        .iter()
        .map(|export| ExportEntry {
            name: export.name.unwrap_or("unknown").to_string(),
            rva: Some(export.rva as u32),
        })
        .collect()
}

/// Extract debug info from PE file
#[cfg(feature = "binary")]
fn extract_pe_debug_info(pe: &PE, data: &[u8]) -> Option<DebugInfo> {
    let mut pdb_path = None;
    let mut cargo_paths = Vec::new();

    // Extract PDB path from debug directory
    if let Some(debug_data) = &pe.debug_data {
        if let Some(pdb70) = debug_data.codeview_pdb70_debug_info {
            // Convert filename from bytes to string
            if let Ok(path) = std::str::from_utf8(pdb70.filename) {
                pdb_path = Some(path.to_string());
            }
        }
    }

    // Look for Cargo registry paths in strings
    for string in extract_strings(data) {
        if string.content.contains(".cargo/registry/src") {
            cargo_paths.push(string.content);
        }
    }

    // Extract build timestamp from PE header
    let build_timestamp = Some(pe.header.coff_header.time_date_stamp as u64);

    Some(DebugInfo {
        pdb_path,
        cargo_paths,
        build_timestamp,
    })
}

/// Extract imports from ELF file (simplified)
#[cfg(feature = "binary")]
fn extract_elf_imports(elf: &Elf) -> Vec<ImportEntry> {
    let mut imports = Vec::new();
    
    // Iterate dynamic symbols - imports have st_value == 0 and are GLOBAL/WEAK
    for sym in &elf.dynsyms {
        if sym.is_import() {
            if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                if !name.is_empty() {
                    imports.push(ImportEntry {
                        name: name.to_string(),
                        library: None,
                    });
                }
            }
        }
    }
    
    imports
}

/// Extract exports from ELF file
#[cfg(feature = "binary")]
fn extract_elf_exports(elf: &Elf) -> Vec<ExportEntry> {
    use goblin::elf::sym::{STB_GLOBAL, STB_WEAK};
    
    elf.dynsyms
        .iter()
        .filter(|sym| {
            // Exports have st_value != 0 and are GLOBAL or WEAK
            sym.st_value != 0 
                && (sym.st_bind() == STB_GLOBAL || sym.st_bind() == STB_WEAK)
        })
        .filter_map(|sym| {
            elf.dynstrtab.get_at(sym.st_name).map(|name: &str| ExportEntry {
                name: name.to_string(),
                rva: Some(sym.st_value as u32),
            })
        })
        .collect()
}

/// Extract imports from Mach-O file
#[cfg(feature = "binary")]
fn extract_macho_imports(macho: &MachO) -> Vec<ImportEntry> {
    let mut imports = Vec::new();

    // macho.imports() returns Result<Vec<Import>>
    if let Ok(imports_vec) = macho.imports() {
        for import in imports_vec {
            imports.push(ImportEntry {
                name: import.name.to_string(),
                library: Some(import.dylib.to_string()),
            });
        }
    }

    imports
}

/// Extract exports from Mach-O file
#[cfg(feature = "binary")]
fn extract_macho_exports(macho: &MachO) -> Vec<ExportEntry> {
    let mut exports = Vec::new();

    // macho.exports() returns Result<Vec<Export>>
    if let Ok(exports_vec) = macho.exports() {
        for export in exports_vec {
            exports.push(ExportEntry {
                name: export.name,
                rva: Some(export.offset as u32),
            });
        }
    }

    exports
}

/// Fallback stub for when binary feature is disabled
#[cfg(not(feature = "binary"))]
pub fn extract_features(_data: &[u8]) -> Result<BinaryFeatures, String> {
    Err("Binary feature not enabled".to_string())
}

#[cfg(not(feature = "binary"))]
fn extract_strings(_data: &[u8]) -> Vec<ExtractedString> {
    Vec::new()
}

#[cfg(not(feature = "binary"))]
fn calculate_entropy(_data: &[u8]) -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_features() {
        let features = BinaryFeatures::empty();
        assert_eq!(features.format, BinaryFormat::Unknown);
        assert!(features.imports.is_empty());
        assert!(features.exports.is_empty());
        assert!(features.strings.is_empty());
        assert!(features.sections.is_empty());
        assert!(features.debug_info.is_none());
    }

    #[test]
    fn test_string_extraction() {
        // Simple ASCII string test
        let data = b"Hello World\x00Test String\x00";
        let strings = extract_strings(data);
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0].content, "Hello World");
        assert_eq!(strings[1].content, "Test String");
    }

    #[test]
    fn test_string_extraction_min_length() {
        // Strings shorter than 4 chars should be ignored
        let data = b"Hi\x00Test\x00Ab\x00";
        let strings = extract_strings(data);
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0].content, "Test");
    }

    #[test]
    fn test_entropy_calculation() {
        // Zero entropy for uniform data
        let uniform = vec![0x41u8; 256];
        let entropy = calculate_entropy(&uniform);
        assert!(entropy < 0.1);

        // High entropy for varied data (all 256 values)
        let random: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let entropy = calculate_entropy(&random);
        assert!(entropy > 7.0);
    }

    #[test]
    fn test_format_detection_pe() {
        let pe_data = vec![0x4D, 0x5A, 0x90, 0x00];  // MZ header
        assert_eq!(detect_format(&pe_data), BinaryFormat::PE);
    }

    #[test]
    fn test_format_detection_elf() {
        let elf_data = vec![0x7F, 0x45, 0x4C, 0x46, 0x02];  // ELF header
        assert_eq!(detect_format(&elf_data), BinaryFormat::ELF);
    }

    #[test]
    fn test_format_detection_macho() {
        // Mach-O magic (little-endian 32-bit: 0xFEEDFACE = [CE, FA, ED, FE])
        let macho_data = vec![0xCE, 0xFA, 0xED, 0xFE, 0x00];
        assert_eq!(detect_format(&macho_data), BinaryFormat::MachO);
    }
}
