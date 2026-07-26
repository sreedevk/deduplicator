use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 4] = b"DDUP";
const VERSION: u8 = 1;
const RECORD_FIXED_LEN: usize = 45;

pub const CACHE_TTL_DAYS: u64 = 30;

pub struct CacheEntry {
    pub size: u64,
    pub mtime: u64,
    pub strict: bool,
    pub hash: u128,
    pub last_seen: u64,
}

pub struct Cache {
    entries: HashMap<PathBuf, CacheEntry>,
    enabled: bool,
}

pub fn mtime_nanos(modified: SystemTime) -> u64 {
    modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

impl Cache {
    pub fn disabled() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: false,
        }
    }

    fn empty_enabled() -> Self {
        Self {
            entries: HashMap::new(),
            enabled: true,
        }
    }

    pub fn load(path: &Path) -> Self {
        match fs::read(path) {
            Ok(bytes) => Self::parse(&bytes).unwrap_or_else(Self::empty_enabled),
            Err(_) => Self::empty_enabled(),
        }
    }

    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 || &bytes[0..4] != MAGIC || bytes[4] != VERSION {
            return None;
        }

        let mut entries = HashMap::new();
        let mut cur = &bytes[5..];

        while cur.len() >= RECORD_FIXED_LEN {
            let hash = u128::from_le_bytes(cur[0..16].try_into().ok()?);
            let size = u64::from_le_bytes(cur[16..24].try_into().ok()?);
            let mtime = u64::from_le_bytes(cur[24..32].try_into().ok()?);
            let last_seen = u64::from_le_bytes(cur[32..40].try_into().ok()?);
            let strict = cur[40] != 0;
            let path_len = u32::from_le_bytes(cur[41..45].try_into().ok()?) as usize;

            let rest = &cur[RECORD_FIXED_LEN..];
            if rest.len() < path_len {
                break;
            }

            match std::str::from_utf8(&rest[..path_len]) {
                Ok(s) => {
                    entries.insert(
                        PathBuf::from(s),
                        CacheEntry {
                            size,
                            mtime,
                            strict,
                            hash,
                            last_seen,
                        },
                    );
                }
                Err(_) => {}
            }

            cur = &rest[path_len..];
        }

        Some(Self {
            entries,
            enabled: true,
        })
    }

    pub fn lookup(&self, path: &Path, size: u64, mtime: u64, strict: bool) -> Option<u128> {
        let entry = self.entries.get(path)?;
        match entry.size == size && entry.mtime == mtime && entry.strict == strict {
            true => Some(entry.hash),
            false => None,
        }
    }

    pub fn record(
        &mut self,
        path: &Path,
        size: u64,
        mtime: u64,
        strict: bool,
        hash: u128,
        now_secs: u64,
    ) {
        if path.to_str().is_none() {
            return;
        }

        self.entries.insert(
            path.to_path_buf(),
            CacheEntry {
                size,
                mtime,
                strict,
                hash,
                last_seen: now_secs,
            },
        );
    }

    pub fn save(&self, path: &Path, ttl_secs: u64, now_secs: u64) {
        if !self.enabled {
            return;
        }

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(MAGIC);
        buf.push(VERSION);

        for (entry_path, entry) in &self.entries {
            if now_secs.saturating_sub(entry.last_seen) > ttl_secs {
                continue;
            }
            let path_str = match entry_path.to_str() {
                Some(s) => s,
                None => continue,
            };

            buf.extend_from_slice(&entry.hash.to_le_bytes());
            buf.extend_from_slice(&entry.size.to_le_bytes());
            buf.extend_from_slice(&entry.mtime.to_le_bytes());
            buf.extend_from_slice(&entry.last_seen.to_le_bytes());
            buf.push(entry.strict as u8);
            buf.extend_from_slice(&(path_str.len() as u32).to_le_bytes());
            buf.extend_from_slice(path_str.as_bytes());
        }

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Err(err) = fs::write(path, &buf) {
            eprintln!("warning: failed to write cache to {}: {err}", path.display());
        }
    }

    pub fn default_path() -> Option<PathBuf> {
        dirs::cache_dir().map(|dir| dir.join("deduplicator").join("cache.bin"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::{Duration, UNIX_EPOCH};
    use tempfile::TempDir;

    #[test]
    fn lookup_hits_on_exact_match() {
        let mut c = Cache::disabled();
        c.record(Path::new("/a/b"), 100, 200, false, 42, 1000);
        assert_eq!(c.lookup(Path::new("/a/b"), 100, 200, false), Some(42));
    }

    #[test]
    fn lookup_misses_on_any_field_change_or_absent() {
        let mut c = Cache::disabled();
        c.record(Path::new("/a/b"), 100, 200, false, 42, 1000);
        assert_eq!(c.lookup(Path::new("/a/b"), 101, 200, false), None);
        assert_eq!(c.lookup(Path::new("/a/b"), 100, 201, false), None);
        assert_eq!(c.lookup(Path::new("/a/b"), 100, 200, true), None);
        assert_eq!(c.lookup(Path::new("/a/x"), 100, 200, false), None);
    }

    #[test]
    fn save_then_load_roundtrips_entries() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cache.bin");
        let mut c = Cache::load(&file);
        c.record(Path::new("/a/b"), 100, 200, false, 42, 1000);
        c.record(Path::new("/c/d"), 5, 6, true, 99, 1000);
        c.save(&file, 86_400, 1000);

        let loaded = Cache::load(&file);
        assert_eq!(loaded.lookup(Path::new("/a/b"), 100, 200, false), Some(42));
        assert_eq!(loaded.lookup(Path::new("/c/d"), 5, 6, true), Some(99));
    }

    #[test]
    fn load_returns_empty_on_corrupt_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cache.bin");
        std::fs::write(&file, b"not a valid cache file at all").unwrap();
        let c = Cache::load(&file);
        assert_eq!(c.lookup(Path::new("/a/b"), 100, 200, false), None);
    }

    #[test]
    fn load_returns_empty_on_version_mismatch() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cache.bin");
        std::fs::write(&file, b"DDUP\x02").unwrap();
        let c = Cache::load(&file);
        assert_eq!(c.lookup(Path::new("/a/b"), 100, 200, false), None);
    }

    #[test]
    fn save_prunes_entries_older_than_ttl() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("cache.bin");
        let mut c = Cache::load(&file);
        c.record(Path::new("/old"), 1, 2, false, 10, 1000);
        c.record(Path::new("/new"), 3, 4, false, 20, 5000);
        c.save(&file, 1000, 5000);

        let loaded = Cache::load(&file);
        assert_eq!(loaded.lookup(Path::new("/old"), 1, 2, false), None);
        assert_eq!(loaded.lookup(Path::new("/new"), 3, 4, false), Some(20));
    }

    #[test]
    fn mtime_nanos_converts_systemtime() {
        let t = UNIX_EPOCH + Duration::from_nanos(1234);
        assert_eq!(mtime_nanos(t), 1234);
    }
}
