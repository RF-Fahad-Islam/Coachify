# 0039. CSV / Excel Export: Rationale

## Key Decisions

### 1. Server-Side (Rust Core) Serialization vs. Client-Side Array Mapping
Generating CSV content in `crates/core/src/reports.rs` directly from SQLite cursors rather than serializing 10,000+ JSON objects to the UI and mapping them in JavaScript avoids IPC serialization overhead and keeps the UI thread 100% responsive even on 4 GB RAM PCs.

### 2. UTF-8 BOM (`\u{FEFF}`) + RFC 4180 CRLF for Excel Compatibility
Microsoft Excel on Windows defaults to legacy ANSI/Windows-1252 codepages when opening `.csv` files unless the 3-byte UTF-8 BOM (`EF BB BF`) is present at byte offset 0. Prepending `\u{FEFF}` guarantees zero-configuration Bangla rendering in Excel 2016/2019/365.
