//! SAG (Segment Address Generator) file parser for Andes RISC-V toolchain
//!
//! Parses `.sag` files and generates equivalent GNU LD linker scripts.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

/// Errors that can occur during SAG parsing
#[derive(Debug, thiserror::Error)]
pub enum SagError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
}

/// Represents an address that can be absolute or relative
#[derive(Debug, Clone)]
pub enum Address {
    /// Absolute address (e.g., `0x80000000`)
    Absolute(u64),
    /// Relative to previous section (e.g., `+0`)
    Relative(i64),
}

impl Address {
    fn parse(s: &str) -> Result<Self, SagError> {
        let s = s.trim();
        if s.starts_with('+') || s.starts_with('-') {
            let val: i64 = if s.starts_with("+") {
                s[1..].trim().parse().map_err(|_| SagError::InvalidAddress(s.to_string()))?
            } else {
                s.parse().map_err(|_| SagError::InvalidAddress(s.to_string()))?
            };
            Ok(Address::Relative(val))
        } else if s.starts_with("0x") || s.starts_with("0X") {
            let val = u64::from_str_radix(&s[2..], 16)
                .map_err(|_| SagError::InvalidAddress(s.to_string()))?;
            Ok(Address::Absolute(val))
        } else {
            let val: u64 = s.parse().map_err(|_| SagError::InvalidAddress(s.to_string()))?;
            Ok(Address::Absolute(val))
        }
    }

    /// Resolve to absolute address given a base
    pub fn resolve(&self, base: u64) -> u64 {
        match self {
            Address::Absolute(addr) => *addr,
            Address::Relative(offset) => (base as i64 + offset) as u64,
        }
    }
}

/// A directive within a region (ADDR, LOADADDR, section pattern, etc.)
#[derive(Debug, Clone)]
pub enum Directive {
    /// `ADDR symbol` or `ADDR NEXT symbol` - define symbol at current VMA
    Addr { symbol: String, next: bool },
    /// `LOADADDR symbol` or `LOADADDR NEXT symbol` - define symbol at current LMA
    LoadAddr { symbol: String, next: bool },
    /// `* ( .section )` or `* KEEP ( .section )` - place sections
    Section { pattern: String, keep: bool },
    /// `STACK = address` - set stack pointer
    Stack(u64),
}

/// A memory region within a block
#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub vma: Address,
    pub directives: Vec<Directive>,
}

/// A section block (HEAD, MEM, LDSECTION, EXEC, DATA)
#[derive(Debug, Clone)]
pub struct Block {
    pub block_type: String,
    pub lma: Address,
    pub alignment: Option<u64>,
    pub regions: Vec<Region>,
}

/// Parsed SAG file
#[derive(Debug, Clone)]
pub struct SagFile {
    pub user_sections: Vec<String>,
    pub blocks: Vec<Block>,
}

/// Parser state machine
struct Parser<'a> {
    lines: Vec<&'a str>,
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines().collect(),
            current: 0,
        }
    }

    fn current_line(&self) -> Option<&'a str> {
        self.lines.get(self.current).copied()
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn line_number(&self) -> usize {
        self.current + 1
    }

    fn skip_empty_and_comments(&mut self) {
        while let Some(line) = self.current_line() {
            let trimmed = line.split(';').next().unwrap_or("").trim();
            if trimmed.is_empty() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_error(&self, message: impl Into<String>) -> SagError {
        SagError::Parse {
            line: self.line_number(),
            message: message.into(),
        }
    }

    fn parse(&mut self) -> Result<SagFile, SagError> {
        let mut user_sections = Vec::new();
        let mut blocks = Vec::new();

        while self.current_line().is_some() {
            self.skip_empty_and_comments();
            let Some(line) = self.current_line() else {
                break;
            };

            // Remove comments
            let line = line.split(';').next().unwrap_or("").trim();
            if line.is_empty() {
                self.advance();
                continue;
            }

            // Parse USER_SECTIONS
            if line.starts_with("USER_SECTIONS") {
                let section = line
                    .strip_prefix("USER_SECTIONS")
                    .ok_or_else(|| self.parse_error("Expected section name after USER_SECTIONS"))?
                    .trim();
                user_sections.push(section.to_string());
                self.advance();
                continue;
            }

            // Parse block (HEAD, MEM, LDSECTION, EXEC, DATA)
            if let Some(block) = self.try_parse_block()? {
                blocks.push(block);
                continue;
            }

            self.advance();
        }

        Ok(SagFile {
            user_sections,
            blocks,
        })
    }

    fn try_parse_block(&mut self) -> Result<Option<Block>, SagError> {
        let line = self.current_line().unwrap();
        let line = line.split(';').next().unwrap_or("").trim();

        // Check for block keywords
        let block_types = ["HEAD", "MEM", "LDSECTION", "EXEC", "DATA"];
        let mut block_type = None;
        let mut rest = line;

        for bt in block_types {
            if line.starts_with(bt) {
                block_type = Some(bt.to_string());
                rest = line[bt.len()..].trim();
                break;
            }
        }

        let Some(block_type) = block_type else {
            return Ok(None);
        };

        // Parse address and optional ALIGN
        let mut parts: Vec<&str> = rest.split_whitespace().collect();

        if parts.is_empty() {
            return Err(self.parse_error("Expected address after block type"));
        }

        let lma = Address::parse(parts[0])?;
        parts.remove(0);

        let mut alignment = None;
        if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("ALIGN") {
            alignment = Some(
                parts[1]
                    .parse::<u64>()
                    .map_err(|_| self.parse_error("Invalid alignment value"))?,
            );
        }

        self.advance();
        self.skip_empty_and_comments();

        // Expect opening brace
        let line = self.current_line().ok_or_else(|| self.parse_error("Expected '{'"))?;
        if !line.trim().starts_with('{') {
            return Err(self.parse_error("Expected '{'"));
        }
        self.advance();

        // Parse regions
        let mut regions = Vec::new();
        loop {
            self.skip_empty_and_comments();
            let Some(line) = self.current_line() else {
                return Err(self.parse_error("Unexpected end of file, expected '}'"));
            };

            let line = line.split(';').next().unwrap_or("").trim();
            if line.starts_with('}') {
                self.advance();
                break;
            }

            // Try to parse a region
            if let Some(region) = self.try_parse_region()? {
                regions.push(region);
            } else {
                self.advance();
            }
        }

        Ok(Some(Block {
            block_type,
            lma,
            alignment,
            regions,
        }))
    }

    fn try_parse_region(&mut self) -> Result<Option<Region>, SagError> {
        let line = self.current_line().unwrap();
        let line = line.split(';').next().unwrap_or("").trim();

        // Region line format: NAME 0xADDRESS
        // Must start with uppercase letter and contain an address
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        // Check if this looks like a region name (uppercase identifier)
        let name = parts[0];
        if !name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) {
            return Ok(None);
        }

        // Skip directive keywords
        if ["ADDR", "LOADADDR", "STACK"].contains(&name) {
            return Ok(None);
        }

        // Try to parse as address
        let vma = match Address::parse(parts[1]) {
            Ok(addr) => addr,
            Err(_) => return Ok(None),
        };

        self.advance();
        self.skip_empty_and_comments();

        // Expect opening brace for region content
        let line = self.current_line().ok_or_else(|| self.parse_error("Expected '{'"))?;
        if !line.trim().starts_with('{') {
            return Err(self.parse_error("Expected '{' after region"));
        }
        self.advance();

        // Parse directives
        let mut directives = Vec::new();
        loop {
            self.skip_empty_and_comments();
            let Some(line) = self.current_line() else {
                return Err(self.parse_error("Unexpected end of file in region"));
            };

            let line = line.split(';').next().unwrap_or("").trim();
            if line.starts_with('}') {
                self.advance();
                break;
            }

            if let Some(directive) = self.parse_directive(line)? {
                directives.push(directive);
            }
            self.advance();
        }

        Ok(Some(Region {
            name: name.to_string(),
            vma,
            directives,
        }))
    }

    fn parse_directive(&self, line: &str) -> Result<Option<Directive>, SagError> {
        let line = line.trim();

        // ADDR [NEXT] symbol
        if line.starts_with("ADDR") {
            let rest = line[4..].trim();
            let (next, symbol) = if rest.starts_with("NEXT") {
                (true, rest[4..].trim().to_string())
            } else {
                (false, rest.to_string())
            };
            return Ok(Some(Directive::Addr { symbol, next }));
        }

        // LOADADDR [NEXT] symbol
        if line.starts_with("LOADADDR") {
            let rest = line[8..].trim();
            let (next, symbol) = if rest.starts_with("NEXT") {
                (true, rest[4..].trim().to_string())
            } else {
                (false, rest.to_string())
            };
            return Ok(Some(Directive::LoadAddr { symbol, next }));
        }

        // STACK = address
        if line.starts_with("STACK") {
            let rest = line[5..].trim();
            let rest = rest.strip_prefix('=').unwrap_or(rest).trim();
            let addr = if rest.starts_with("0x") || rest.starts_with("0X") {
                u64::from_str_radix(&rest[2..], 16)
                    .map_err(|_| self.parse_error("Invalid stack address"))?
            } else {
                rest.parse().map_err(|_| self.parse_error("Invalid stack address"))?
            };
            return Ok(Some(Directive::Stack(addr)));
        }

        // * [KEEP] ( sections )
        if line.starts_with('*') {
            let rest = line[1..].trim();
            let keep = rest.starts_with("KEEP");
            let rest = if keep { rest[4..].trim() } else { rest };

            // Extract content between parentheses
            if let Some(start) = rest.find('(') {
                if let Some(end) = rest.rfind(')') {
                    let pattern = rest[start + 1..end].trim().to_string();
                    return Ok(Some(Directive::Section { pattern, keep }));
                }
            }
        }

        Ok(None)
    }
}

impl SagFile {
    /// Parse a SAG file from a string
    pub fn parse(content: &str) -> Result<Self, SagError> {
        let mut parser = Parser::new(content);
        parser.parse()
    }

    /// Parse a SAG file from a path
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, SagError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Generate a GNU LD linker script
    pub fn to_linker_script(&self, config: &LinkerScriptConfig) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "/* Auto-generated from SAG file */").unwrap();
        writeln!(output, "/* Config: {:?} */", config.name).unwrap();
        writeln!(output).unwrap();
        writeln!(output, "OUTPUT_ARCH(riscv)").unwrap();
        writeln!(output, "ENTRY(_start)").unwrap();
        writeln!(output).unwrap();

        // Memory regions
        writeln!(output, "MEMORY").unwrap();
        writeln!(output, "{{").unwrap();
        for (name, region) in &config.memory_regions {
            writeln!(
                output,
                "    {} ({}) : ORIGIN = {:#010X}, LENGTH = {}",
                name, region.attributes, region.origin, format_size(region.length)
            )
            .unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        // Stack symbol
        if let Some(stack) = self.find_stack() {
            writeln!(output, "__stack_top = {:#010X};", stack).unwrap();
            writeln!(output).unwrap();
        }

        // Sections
        writeln!(output, "SECTIONS").unwrap();
        writeln!(output, "{{").unwrap();

        let mut current_lma: u64 = 0;

        for block in &self.blocks {
            let block_lma = block.lma.resolve(current_lma);
            if let Some(align) = block.alignment {
                current_lma = (block_lma + align - 1) & !(align - 1);
            } else {
                current_lma = block_lma;
            }

            writeln!(output).unwrap();
            writeln!(output, "    /* Block: {} @ LMA {:#010X} */", block.block_type, current_lma).unwrap();

            for region in &block.regions {
                let vma = region.vma.resolve(0);
                self.emit_region(&mut output, region, current_lma, vma, config);
            }
        }

        // Standard end symbols
        writeln!(output).unwrap();
        writeln!(output, "    PROVIDE(_end = .);").unwrap();
        writeln!(output, "    PROVIDE(end = .);").unwrap();
        writeln!(output, "}}").unwrap();

        output
    }

    fn emit_region(
        &self,
        output: &mut String,
        region: &Region,
        lma: u64,
        vma: u64,
        config: &LinkerScriptConfig,
    ) {
        writeln!(output).unwrap();
        writeln!(output, "    /* Region: {} VMA={:#010X} LMA={:#010X} */", region.name, vma, lma).unwrap();

        // Determine memory region name
        let mem_region = config.vma_to_region(vma).unwrap_or("RAM");

        // Check if this runs in place (VMA == LMA region)
        let runs_in_place = config.vma_to_region(vma) == config.vma_to_region(lma);

        for directive in &region.directives {
            match directive {
                Directive::Addr { symbol, .. } => {
                    writeln!(output, "    {} = .;", symbol).unwrap();
                }
                Directive::LoadAddr { symbol, .. } => {
                    writeln!(output, "    {} = LOADADDR(.{});", symbol, region.name.to_lowercase()).unwrap();
                }
                Directive::Section { pattern, keep } => {
                    let sections = self.expand_section_pattern(pattern);
                    for section in sections {
                        let keep_str = if *keep { "KEEP" } else { "" };
                        if runs_in_place {
                            writeln!(output, "    .{} :", section).unwrap();
                        } else {
                            writeln!(output, "    .{} : AT({})", section, lma).unwrap();
                        }
                        writeln!(output, "    {{").unwrap();
                        if *keep {
                            writeln!(output, "        {}(*(.{}))", keep_str, section).unwrap();
                            writeln!(output, "        {}(*(.{}*))", keep_str, section).unwrap();
                        } else {
                            writeln!(output, "        *(.{})", section).unwrap();
                            writeln!(output, "        *(.{}*)", section).unwrap();
                        }
                        writeln!(output, "    }} > {}", mem_region).unwrap();
                    }
                }
                Directive::Stack(addr) => {
                    writeln!(output, "    __stack_top = {:#010X};", addr).unwrap();
                }
            }
        }
    }

    fn expand_section_pattern(&self, pattern: &str) -> Vec<String> {
        let mut sections = Vec::new();

        for part in pattern.split(',') {
            let part = part.trim();
            match part {
                "+ISR" => sections.extend(["vectors", "isr"].map(String::from)),
                "+RO" => sections.extend(["text", "rodata", "srodata"].map(String::from)),
                "+RW" => sections.extend(["data", "sdata"].map(String::from)),
                "+ZI" => sections.extend(["bss", "sbss"].map(String::from)),
                s if s.starts_with('.') => sections.push(s[1..].to_string()),
                s => sections.push(s.to_string()),
            }
        }

        sections
    }

    fn find_stack(&self) -> Option<u64> {
        for block in &self.blocks {
            for region in &block.regions {
                for directive in &region.directives {
                    if let Directive::Stack(addr) = directive {
                        return Some(*addr);
                    }
                }
            }
        }
        None
    }
}

/// Memory region configuration
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub origin: u64,
    pub length: u64,
    pub attributes: String,
}

/// Configuration for linker script generation
#[derive(Debug, Clone)]
pub struct LinkerScriptConfig {
    pub name: String,
    pub memory_regions: HashMap<String, MemoryRegion>,
}

impl LinkerScriptConfig {
    /// Create a default config for AE350 DDR mode
    pub fn ae350_ddr() -> Self {
        let mut memory_regions = HashMap::new();

        memory_regions.insert(
            "FLASH".to_string(),
            MemoryRegion {
                origin: 0x80000000,
                length: 256 * 1024 * 1024,
                attributes: "rx".to_string(),
            },
        );

        memory_regions.insert(
            "DDR".to_string(),
            MemoryRegion {
                origin: 0x00000000,
                length: 128 * 1024 * 1024,
                attributes: "rwx".to_string(),
            },
        );

        Self {
            name: "AE350 DDR".to_string(),
            memory_regions,
        }
    }

    /// Create a default config for AE350 ILM mode
    pub fn ae350_ilm() -> Self {
        let mut memory_regions = HashMap::new();

        memory_regions.insert(
            "FLASH".to_string(),
            MemoryRegion {
                origin: 0x80000000,
                length: 256 * 1024 * 1024,
                attributes: "rx".to_string(),
            },
        );

        memory_regions.insert(
            "ILM".to_string(),
            MemoryRegion {
                origin: 0xA0000000,
                length: 2 * 1024 * 1024,
                attributes: "rwx".to_string(),
            },
        );

        Self {
            name: "AE350 ILM".to_string(),
            memory_regions,
        }
    }

    fn vma_to_region(&self, vma: u64) -> Option<&str> {
        for (name, region) in &self.memory_regions {
            if vma >= region.origin && vma < region.origin + region.length {
                return Some(name);
            }
        }
        None
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 && bytes % (1024 * 1024 * 1024) == 0 {
        format!("{}G", bytes / (1024 * 1024 * 1024))
    } else if bytes >= 1024 * 1024 && bytes % (1024 * 1024) == 0 {
        format!("{}M", bytes / (1024 * 1024))
    } else if bytes >= 1024 && bytes % 1024 == 0 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{}", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        assert!(matches!(Address::parse("0x80000000").unwrap(), Address::Absolute(0x80000000)));
        assert!(matches!(Address::parse("+0").unwrap(), Address::Relative(0)));
        assert!(matches!(Address::parse("+256").unwrap(), Address::Relative(256)));
    }

    #[test]
    fn test_parse_simple_sag() {
        let content = r#"
USER_SECTIONS .bootloader

HEAD 0x00000000
{
    BOOTLOADER 0x80000000
    {
        ADDR __flash_start
        * KEEP ( .bootloader )
    }
}
"#;
        let sag = SagFile::parse(content).unwrap();
        assert_eq!(sag.user_sections.len(), 1);
        assert_eq!(sag.blocks.len(), 1);
        assert_eq!(sag.blocks[0].regions.len(), 1);
    }
}
