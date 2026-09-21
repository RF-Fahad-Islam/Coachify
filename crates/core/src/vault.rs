use crate::crypto;
use crate::db;
use crate::storage;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader, Limits};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub const MAX_FILE_SIZE_BYTES: usize = 20 * 1024 * 1024; // 20 MB
pub const MAX_IMAGE_DIMENSION: u32 = 6000; // 6000 px
pub const MAX_THUMBNAIL_DIMENSION: u32 = 800; // 800 px

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    #[error("File size ({size} bytes) exceeds maximum limit of 20MB")]
    FileTooLarge { size: usize },
    #[error("File data cannot be empty")]
    EmptyData,
    #[error("Image decode failed: {0}")]
    ImageDecodeFailed(String),
    #[error("Image dimensions ({width}x{height}) exceed maximum allowed limit of 6000x6000")]
    DimensionsExceeded { width: u32, height: u32 },
    #[error("Base64 decode error: {0}")]
    Base64Decode(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Unsupported file format: {0}. Only JPEG, PNG, WebP, and PDF are supported.")]
    UnsupportedFormat(String),
}

impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        VaultError::Io(err.to_string())
    }
}

impl From<rusqlite::Error> for VaultError {
    fn from(err: rusqlite::Error) -> Self {
        VaultError::Sql(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct DocumentSummary {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub kind: String,
    pub original_name: String,
    pub mime: String,
    pub size: i64,
    pub sha256: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UploadFilePayload {
    pub owner_type: String,
    pub owner_id: String,
    pub kind: String,
    pub file_name: String,
    pub mime_type: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetFileResponse {
    pub data_base64: String,
    pub mime_type: String,
    pub file_name: String,
}

/// Resolves the local institution vault storage directory.
pub fn resolve_vault_dir() -> PathBuf {
    if let Ok(Some(cfg)) = storage::load_global_config() {
        let p = PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("vault");
        return p;
    }

    if let Ok(dev_db) = db::resolve_dev_db_path() {
        if let Some(parent) = dev_db.parent() {
            return parent.join("vault");
        }
    }

    PathBuf::from("vault")
}

/// Computes SHA256 hexadecimal string for binary slice.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Extracts file extension from a filename or mime type.
pub fn get_file_extension(filename: &str, mime: &str) -> String {
    if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
        let clean = ext.to_lowercase();
        if !clean.is_empty() && clean.len() <= 5 {
            return clean;
        }
    }

    match mime {
        "image/jpeg" | "image/jpg" => "jpg".to_string(),
        "image/png" => "png".to_string(),
        "image/webp" => "webp".to_string(),
        "application/pdf" => "pdf".to_string(),
        _ => "bin".to_string(),
    }
}

/// Decodes, validates, resizes (if image), and saves a file in the local vault.
pub fn save_to_vault(
    vault_dir: &Path,
    filename: &str,
    mime_type: &str,
    raw_bytes: &[u8],
) -> Result<(String, String, i64, String), VaultError> {
    if raw_bytes.is_empty() {
        return Err(VaultError::EmptyData);
    }
    if raw_bytes.len() > MAX_FILE_SIZE_BYTES {
        return Err(VaultError::FileTooLarge {
            size: raw_bytes.len(),
        });
    }

    let is_image = mime_type.starts_with("image/")
        || filename.ends_with(".jpg")
        || filename.ends_with(".jpeg")
        || filename.ends_with(".png")
        || filename.ends_with(".webp");

    let is_pdf = mime_type == "application/pdf" || filename.ends_with(".pdf");
    if !is_image && !is_pdf {
        return Err(VaultError::UnsupportedFormat(mime_type.to_string()));
    }

    let (final_bytes, final_mime, ext) = if is_image {
        let mut reader = ImageReader::new(Cursor::new(raw_bytes))
            .with_guessed_format()
            .map_err(|e| {
                VaultError::ImageDecodeFailed(format!("Failed to guess image format: {}", e))
            })?;
        let guessed_format = reader.format();

        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
        limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
        limits.max_alloc = Some(MAX_FILE_SIZE_BYTES as u64);
        reader.limits(limits);

        let dynamic_img = reader.decode().map_err(|e| {
            let msg = e.to_string();
            if msg.contains("limit") || msg.contains("dimension") {
                VaultError::DimensionsExceeded {
                    width: MAX_IMAGE_DIMENSION + 1,
                    height: MAX_IMAGE_DIMENSION + 1,
                }
            } else {
                VaultError::ImageDecodeFailed(msg)
            }
        })?;

        let (orig_w, orig_h) = (dynamic_img.width(), dynamic_img.height());
        if orig_w > MAX_IMAGE_DIMENSION || orig_h > MAX_IMAGE_DIMENSION {
            return Err(VaultError::DimensionsExceeded {
                width: orig_w,
                height: orig_h,
            });
        }

        // Downscale if larger than 800x800
        let (processed_img, out_format, out_mime, out_ext) =
            if orig_w > MAX_THUMBNAIL_DIMENSION || orig_h > MAX_THUMBNAIL_DIMENSION {
                let resized = dynamic_img.resize(
                    MAX_THUMBNAIL_DIMENSION,
                    MAX_THUMBNAIL_DIMENSION,
                    FilterType::Lanczos3,
                );
                (resized, ImageFormat::Jpeg, "image/jpeg", "jpg")
            } else {
                let fmt = guessed_format.unwrap_or(ImageFormat::Jpeg);
                let (m, e) = match fmt {
                    ImageFormat::Png => ("image/png", "png"),
                    ImageFormat::WebP => ("image/webp", "webp"),
                    _ => ("image/jpeg", "jpg"),
                };
                (dynamic_img, fmt, m, e)
            };

        let mut out_buffer = Vec::new();
        processed_img
            .write_to(&mut Cursor::new(&mut out_buffer), out_format)
            .map_err(|e| {
                VaultError::ImageDecodeFailed(format!("Failed to re-encode image: {}", e))
            })?;

        (out_buffer, out_mime.to_string(), out_ext.to_string())
    } else {
        // Non-image files stored verbatim
        let ext = get_file_extension(filename, mime_type);
        (raw_bytes.to_vec(), mime_type.to_string(), ext)
    };

    let digest = compute_sha256(&final_bytes);
    let prefix = &digest[0..2];
    let target_dir = vault_dir.join(prefix);
    fs::create_dir_all(&target_dir)?;

    let target_file = target_dir.join(format!("{}.{}", digest, ext));
    fs::write(&target_file, &final_bytes)?;

    let file_size = final_bytes.len() as i64;
    Ok((digest, final_mime, file_size, ext))
}

/// Uploads a file payload into the vault and records it in the `documents` table.
pub fn upload_file(
    conn: &Connection,
    vault_dir: &Path,
    payload: &UploadFilePayload,
    user_id: Option<&str>,
    now: &str,
) -> Result<DocumentSummary, VaultError> {
    // Strip data URL scheme prefix if present (e.g. data:image/jpeg;base64,...)
    let b64_clean = if let Some(idx) = payload.data_base64.find(";base64,") {
        &payload.data_base64[idx + 8..]
    } else {
        payload.data_base64.trim()
    };

    let raw_bytes =
        crypto::base64_decode(b64_clean).map_err(|e| VaultError::Base64Decode(format!("{}", e)))?;

    let (digest, final_mime, file_size, _ext) = save_to_vault(
        vault_dir,
        &payload.file_name,
        &payload.mime_type,
        &raw_bytes,
    )?;

    let doc_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO documents (
            id, owner_type, owner_id, kind, original_name, mime, size, sha256, uploaded_by, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            doc_id,
            payload.owner_type,
            payload.owner_id,
            payload.kind,
            payload.file_name,
            final_mime,
            file_size,
            digest,
            user_id,
            now,
        ],
    )?;

    Ok(DocumentSummary {
        id: doc_id,
        owner_type: payload.owner_type.clone(),
        owner_id: payload.owner_id.clone(),
        kind: payload.kind.clone(),
        original_name: payload.file_name.clone(),
        mime: final_mime,
        size: file_size,
        sha256: digest,
        created_at: now.to_string(),
    })
}

/// Retrieves a file from the vault by document ID, returning base64 encoded content.
pub fn get_file(
    conn: &Connection,
    vault_dir: &Path,
    document_id: &str,
) -> Result<GetFileResponse, VaultError> {
    let mut stmt =
        conn.prepare("SELECT original_name, mime, sha256 FROM documents WHERE id = ?1 LIMIT 1")?;
    let mut rows = stmt.query(rusqlite::params![document_id])?;

    let (original_name, mime, sha256) = if let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let mime: String = row.get(1)?;
        let sha256: String = row.get(2)?;
        (name, mime, sha256)
    } else {
        return Err(VaultError::NotFound(document_id.to_string()));
    };

    let prefix = &sha256[0..2];
    let folder = vault_dir.join(prefix);

    // Locate file matching <sha256>.*
    let mut found_path: Option<PathBuf> = None;
    if folder.exists() {
        if let Ok(entries) = fs::read_dir(&folder) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&sha256) {
                    found_path = Some(entry.path());
                    break;
                }
            }
        }
    }

    let file_path = found_path.ok_or_else(|| {
        VaultError::NotFound(format!(
            "Physical file for document {} not found in vault",
            document_id
        ))
    })?;

    let bytes = fs::read(&file_path)?;
    let b64 = crypto::base64_encode(&bytes);

    Ok(GetFileResponse {
        data_base64: b64,
        mime_type: mime,
        file_name: original_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_vault_text_and_pdf_storage() {
        let dir = tempdir().expect("tempdir");
        let content = b"%PDF-1.4 mock pdf document content";
        let (digest, mime, size, ext) =
            save_to_vault(dir.path(), "certificate.pdf", "application/pdf", content).unwrap();

        assert_eq!(ext, "pdf");
        assert_eq!(mime, "application/pdf");
        assert_eq!(size, content.len() as i64);

        let target = dir
            .path()
            .join(&digest[0..2])
            .join(format!("{}.pdf", digest));
        assert!(target.exists());
        let read_back = fs::read(target).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn test_vault_image_resize_downscales() {
        let dir = tempdir().expect("tempdir");

        // Create a 1200x900 test image in memory
        let img = image::RgbImage::new(1200, 900);
        let mut jpeg_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut jpeg_bytes), ImageFormat::Jpeg)
            .unwrap();

        let (digest, mime, size, ext) =
            save_to_vault(dir.path(), "photo.jpg", "image/jpeg", &jpeg_bytes).unwrap();

        assert_eq!(ext, "jpg");
        assert_eq!(mime, "image/jpeg");
        assert!(size > 0);

        let target = dir
            .path()
            .join(&digest[0..2])
            .join(format!("{}.jpg", digest));
        assert!(target.exists());

        // Verify decoded image is at most 800x800
        let saved_bytes = fs::read(target).unwrap();
        let decoded = image::load_from_memory(&saved_bytes).unwrap();
        assert!(decoded.width() <= 800);
        assert!(decoded.height() <= 800);
    }

    #[test]
    fn test_vault_rejects_oversized_file() {
        let dir = tempdir().expect("tempdir");
        let oversized = vec![0u8; MAX_FILE_SIZE_BYTES + 10];
        let result = save_to_vault(
            dir.path(),
            "big.bin",
            "application/octet-stream",
            &oversized,
        );
        assert!(matches!(result, Err(VaultError::FileTooLarge { .. })));
    }
}
