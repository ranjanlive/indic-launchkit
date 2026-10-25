use crate::{hash_file, Result, MAX_APK};
use serde::Serialize;
use std::{fs, io::Read, path::Path};

#[derive(Debug, Clone, Serialize, Default)]
pub struct ApkInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<u64>,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub bytes: u64,
    pub sha256: String,
    pub signing_status: String,
}
fn u16_at(b: &[u8], p: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2)
            .ok_or("Truncated binary XML")?
            .try_into()
            .unwrap(),
    ))
}
fn u32_at(b: &[u8], p: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4)
            .ok_or("Truncated binary XML")?
            .try_into()
            .unwrap(),
    ))
}
fn len8(b: &[u8], p: &mut usize) -> Result<usize> {
    let a = *b.get(*p).ok_or("Truncated string length")?;
    *p += 1;
    if a & 0x80 == 0 {
        Ok(a as usize)
    } else {
        let c = *b.get(*p).ok_or("Truncated string length")?;
        *p += 1;
        Ok((((a & 0x7f) as usize) << 8) | c as usize)
    }
}
fn len16(b: &[u8], p: &mut usize) -> Result<usize> {
    let a = u16_at(b, *p)?;
    *p += 2;
    if a & 0x8000 == 0 {
        Ok(a as usize)
    } else {
        let c = u16_at(b, *p)?;
        *p += 2;
        Ok((((a & 0x7fff) as usize) << 16) | c as usize)
    }
}
fn pool(chunk: &[u8]) -> Result<Vec<String>> {
    let header = u16_at(chunk, 2)? as usize;
    let count = u32_at(chunk, 8)? as usize;
    let flags = u32_at(chunk, 16)?;
    let start = u32_at(chunk, 20)? as usize;
    if count > 100_000
        || header < 28
        || header
            .checked_add(count * 4)
            .ok_or("String table overflow")?
            > chunk.len()
    {
        return Err("Invalid string table".into());
    }
    let mut result = Vec::with_capacity(count);
    let mut decoded = 0usize;
    for i in 0..count {
        let mut p = start
            .checked_add(u32_at(chunk, header + i * 4)? as usize)
            .ok_or("String offset overflow")?;
        if flags & 0x100 != 0 {
            let _utf16 = len8(chunk, &mut p)?;
            let n = len8(chunk, &mut p)?;
            decoded += n;
            if decoded > 16 * 1024 * 1024 {
                return Err("Decoded string table exceeds limit".into());
            }
            let bytes = chunk
                .get(p..p.checked_add(n).ok_or("String overflow")?)
                .ok_or("Truncated UTF-8 string")?;
            result.push(
                std::str::from_utf8(bytes)
                    .map_err(|_| "Invalid UTF-8 string")?
                    .into(),
            );
        } else {
            let n = len16(chunk, &mut p)?;
            decoded += n * 3;
            if n > 1_000_000 || decoded > 16 * 1024 * 1024 {
                return Err("Oversized decoded string table".into());
            }
            let bytes = chunk
                .get(p..p.checked_add(n * 2).ok_or("String overflow")?)
                .ok_or("Truncated UTF-16 string")?;
            let words = bytes
                .chunks_exact(2)
                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                .collect::<Vec<_>>();
            result.push(String::from_utf16(&words).map_err(|_| "Invalid UTF-16 string")?);
        }
    }
    Ok(result)
}
pub fn parse_manifest(bytes: &[u8]) -> Result<ApkInfo> {
    if bytes.len() < 8 || u16_at(bytes, 0)? != 3 {
        return Err("AndroidManifest.xml is not Android binary XML".into());
    }
    let total = u32_at(bytes, 4)? as usize;
    if total != bytes.len() {
        return Err("Binary XML size mismatch".into());
    }
    let mut offset = u16_at(bytes, 2)? as usize;
    if offset < 8 {
        return Err("Invalid binary XML header".into());
    }
    let mut strings: Option<Vec<String>> = None;
    let mut info = ApkInfo::default();
    let mut found = false;
    while offset < total {
        let kind = u16_at(bytes, offset)?;
        let header = u16_at(bytes, offset + 2)? as usize;
        let size = u32_at(bytes, offset + 4)? as usize;
        if header < 8 || size < header || size > total - offset {
            return Err("Invalid binary XML chunk".into());
        }
        let chunk = &bytes[offset..offset + size];
        if kind == 1 {
            if strings.is_some() {
                return Err("Duplicate string table".into());
            }
            strings = Some(pool(chunk)?);
        }
        if kind == 0x102 {
            let strings = strings.as_ref().ok_or("Element before string table")?;
            let get = |i: u32| -> Result<&str> {
                strings
                    .get(i as usize)
                    .map(|s| s.as_str())
                    .ok_or("Invalid string index".into())
            };
            if header < 16 || size < header + 20 {
                return Err("Truncated element".into());
            }
            let element = get(u32_at(chunk, header + 4)?)?;
            let start = u16_at(chunk, header + 8)? as usize;
            let stride = u16_at(chunk, header + 10)? as usize;
            let count = u16_at(chunk, header + 12)? as usize;
            if start < 20 || stride < 20 || count > 4096 || header + start + count * stride > size {
                return Err("Invalid attributes".into());
            }
            if element == "manifest" {
                if found {
                    return Err("Multiple manifest elements".into());
                }
                found = true;
            }
            for i in 0..count {
                let p = header + start + i * stride;
                let name = get(u32_at(chunk, p + 4)?)?;
                let raw = u32_at(chunk, p + 8)?;
                let value_type = *chunk.get(p + 15).ok_or("Truncated typed attribute")?;
                let value = u32_at(chunk, p + 16)?;
                let text = if raw != u32::MAX {
                    Some(get(raw)?.to_string())
                } else if value_type == 3 {
                    Some(get(value)?.to_string())
                } else {
                    None
                };
                let number = if [0x10, 0x11].contains(&value_type) {
                    Some(value)
                } else {
                    text.as_ref().and_then(|t| t.parse().ok())
                };
                match (element, name) {
                    ("manifest", "package") => {
                        info.package_name = text.ok_or("Package name is not a literal string")?
                    }
                    ("manifest", "versionName") => info.version_name = text,
                    ("manifest", "versionCode") => info.version_code = number.map(u64::from),
                    ("uses-sdk", "minSdkVersion") => info.min_sdk = number,
                    ("uses-sdk", "targetSdkVersion") => info.target_sdk = number,
                    _ => {}
                }
            }
        }
        offset += size;
    }
    if !found || info.package_name.is_empty() {
        return Err("No package name found in Android manifest".into());
    }
    if !info
        .package_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
    {
        return Err("Invalid package name".into());
    }
    Ok(info)
}
pub fn inspect(path: &Path) -> Result<ApkInfo> {
    let bytes = fs::metadata(path).map_err(|e| e.to_string())?.len();
    if bytes > MAX_APK {
        return Err("APK exceeds 1 GiB".into());
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("Invalid APK ZIP: {e}"))?;
    if zip.len() > 100_000 {
        return Err("Too many APK entries".into());
    }
    let mut manifest = zip
        .by_name("AndroidManifest.xml")
        .map_err(|_| "APK has no AndroidManifest.xml")?;
    if manifest.size() > 8 * 1024 * 1024 {
        return Err("Manifest exceeds 8 MiB".into());
    }
    let mut data = vec![];
    manifest
        .by_ref()
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 8 * 1024 * 1024 {
        return Err("Decompressed manifest exceeds limit".into());
    }
    let mut info = parse_manifest(&data)?;
    info.bytes = bytes;
    info.sha256 = hash_file(path)?;
    info.signing_status = "not-verified".into();
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_and_hostile_chunks() {
        for n in 0..128 {
            assert!(parse_manifest(&vec![0; n]).is_err())
        }
        let mut b = vec![3, 0, 8, 0, 16, 0, 0, 0, 1, 0, 8, 0, 255, 255, 255, 255];
        assert!(parse_manifest(&b).is_err());
        b[4] = 255;
        assert!(parse_manifest(&b).is_err());
    }
}
