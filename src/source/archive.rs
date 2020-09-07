use std::{
    fmt, fs, io,
    path::{Path, PathBuf, Component},
    process::{Command, Stdio},
};

use crate::cache;

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveFormat {
    extension_length: usize,
    kind: ArchiveFormatKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormatKind {
    Z,
    Zip,
    Gzip,
    Bzip2,
    Lz,
    Xz,
    Lzma,
    P7z,
    Tar,
    TarZ,
    TarGzip,
    TarBzip2,
    TarLz,
    TarXz,
    TarLzma,
    Tar7z,
    TarZstd,
    Rar,
    Zstd,
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ArchiveFormatKind::*;
        let format = match self.kind {
            Z => "z",
            Zip => "zip",
            Gzip => "gzip",
            Bzip2 => "bzip2",
            Lz => "lz",
            Xz => "xz",
            Lzma => "lzma",
            P7z => "7z",
            Tar => "tar",
            TarZ => "tar.z",
            TarGzip => "tar.gzip",
            TarBzip2 => "tar.bzip2",
            TarLz => "tar.lz",
            TarXz => "tar.xz",
            TarLzma => "tar.lzma",
            Tar7z => "tar.7z",
            TarZstd => "tar.zstd",
            Rar => "rar",
            Zstd => "zstd",
        };
        f.write_str(format)
    }
}

pub fn detect(file_name: &str) -> Option<ArchiveFormat> {
    ArchiveFormat::detect(file_name)
}

impl ArchiveFormat {
    fn new(extension_length: usize, kind: ArchiveFormatKind) -> Self {
        Self {
            extension_length,
            kind,
        }
    }
    fn detect(file_name: &str) -> Option<Self> {
        use ArchiveFormatKind::*;
        let file_name = file_name.to_ascii_lowercase();

        if file_name.ends_with(".tar.z") {
            Some(ArchiveFormat::new(6, TarZ))
        } else if file_name.ends_with(".tar.gz") {
            Some(ArchiveFormat::new(7, TarGzip))
        } else if file_name.ends_with(".tgz") {
            Some(ArchiveFormat::new(4, TarGzip))
        } else if file_name.ends_with(".tar.bz2") {
            Some(ArchiveFormat::new(8, TarBzip2))
        } else if file_name.ends_with(".tbz2") {
            Some(ArchiveFormat::new(5, TarBzip2))
        } else if file_name.ends_with(".tar.lz") {
            Some(ArchiveFormat::new(7, TarLz))
        } else if file_name.ends_with(".tar.xz") {
            Some(ArchiveFormat::new(7, TarXz))
        } else if file_name.ends_with(".txz") {
            Some(ArchiveFormat::new(4, TarXz))
        } else if file_name.ends_with(".tar.lzma") {
            Some(ArchiveFormat::new(9, TarLzma))
        } else if file_name.ends_with(".tlz") {
            Some(ArchiveFormat::new(4, TarLzma))
        } else if file_name.ends_with(".tar.7z") {
            Some(ArchiveFormat::new(7, Tar7z))
        } else if file_name.ends_with(".tar.7z.001") {
            Some(ArchiveFormat::new(11, Tar7z))
        } else if file_name.ends_with(".t7z") {
            Some(ArchiveFormat::new(4, Tar7z))
        } else if file_name.ends_with(".tar.zst") {
            Some(ArchiveFormat::new(8, TarZstd))
        } else if file_name.ends_with(".tar") {
            Some(ArchiveFormat::new(4, Tar))
        } else if file_name.ends_with(".z") {
            Some(ArchiveFormat::new(2, Z))
        } else if file_name.ends_with(".zip") {
            Some(ArchiveFormat::new(4, Zip))
        } else if file_name.ends_with(".gz") {
            Some(ArchiveFormat::new(3, Gzip))
        } else if file_name.ends_with(".bz2") {
            Some(ArchiveFormat::new(4, Bzip2))
        } else if file_name.ends_with(".lz") {
            Some(ArchiveFormat::new(3, Lz))
        } else if file_name.ends_with(".xz") {
            Some(ArchiveFormat::new(3, Xz))
        } else if file_name.ends_with(".lzma") {
            Some(ArchiveFormat::new(5, Lzma))
        } else if file_name.ends_with(".7z") {
            Some(ArchiveFormat::new(3, P7z))
        } else if file_name.ends_with(".7z.001") {
            Some(ArchiveFormat::new(7, P7z))
        } else if file_name.ends_with(".rar") {
            Some(ArchiveFormat::new(4, Rar))
        } else if file_name.ends_with(".zst") {
            Some(ArchiveFormat::new(4, Zstd))
        } else {
            None
        }
    }

    pub fn is_tar(&self) -> bool {
        use ArchiveFormatKind::*;
        self.kind == Tar
            || self.kind == TarZ
            || self.kind == TarGzip
            || self.kind == TarBzip2
            || self.kind == TarLz
            || self.kind == TarXz
            || self.kind == TarLzma
            || self.kind == Tar7z
            || self.kind == TarZstd
    }

    pub fn kind(&self) -> ArchiveFormatKind {
        self.kind
    }
}

fn path_for_extraction(path: &Path, archive: &ArchiveFormat, cache_folder: &Path) -> Result<PathBuf> {
    anyhow::ensure!(
        archive.is_tar(),
        "archive format not supported: {}",
        archive
    );
    let head = path.file_name()
        .expect("received a path with no file name")
        .to_string_lossy();
    let head = &head[..head.len() - archive.extension_length];
    let parent = path.parent()
        .expect("received a path with no parent")
        .components()
        .filter(|component| matches!(component, Component::Normal(_)))
        .fold(PathBuf::new(), |p, c| p.join(c));
    let to_encode_path = parent.join(head);

    let encoded_path = cache::to_cached_path(&to_encode_path.to_string_lossy().as_bytes());
    trace!("encoded_path: {}", encoded_path);

    Ok(cache_folder.join(encoded_path))
}

fn extract_archive_to(path: &str, archive: &ArchiveFormat, extract_path: &Path) -> Result<()> {
    assert!(archive.is_tar(), "only tar archives are supported");
    let status = Command::new("tar")
        .arg("xf")
        .arg(path)
        .arg("-C")
        .arg(extract_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to run tar command, is tar installed?")?;
    anyhow::ensure!(
        status.success(),
        "tar command failed to run: `tar xf '{}' -C '{}'",
        path,
        extract_path.display()
    );
    Ok(())
}

pub fn extract(archive_path: &str, archive: &ArchiveFormat, cache_folder: &Path) -> Result<PathBuf> {
    let full_archive_path = Path::new(archive_path)
        .canonicalize()
        .context("failed to canonicalize path of archive")?;
    let extracted_path = path_for_extraction(&full_archive_path, archive, cache_folder)
        .context("failed to deduce extracted path for archive")?;
    debug!("path for extracted archive: {}", extracted_path.display());
    match fs::metadata(&extracted_path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(&extracted_path).with_context(|| {
                    format!(
                        "failed to remove old directory: {}",
                        extracted_path.display()
                    )
                })?;
            } else if metadata.is_file() {
                fs::remove_file(&extracted_path).with_context(|| {
                    format!(
                        "failed to remove old directory: {}",
                        extracted_path.display()
                    )
                })?;
            }
        }
        Err(e) => {
            anyhow::ensure!(
                e.kind() == io::ErrorKind::NotFound,
                "failed to get metadata for path: {}",
                extracted_path.display()
            );
        }
    }
    fs::create_dir(&extracted_path).context("failed to create folder for extraction")?;
    extract_archive_to(&archive_path, archive, &extracted_path).context("failed to extract archive")?;
    Ok(extracted_path)
}

#[cfg(test)]
mod test {
    use super::*;
    use ArchiveFormatKind::*;

    #[test]
    fn parses_correctly() {
        assert_eq!(detect("file.tar.z"), Some(ArchiveFormat::new(6, TarZ)));
        assert_eq!(detect("file.tar.gz"), Some(ArchiveFormat::new(7, TarGzip)));
        assert_eq!(detect("file.tgz"), Some(ArchiveFormat::new(4, TarGzip)));
        assert_eq!(
            detect("file.tar.bz2"),
            Some(ArchiveFormat::new(8, TarBzip2))
        );
        assert_eq!(detect("file.tbz2"), Some(ArchiveFormat::new(5, TarBzip2)));
        assert_eq!(detect("file.tar.lz"), Some(ArchiveFormat::new(7, TarLz)));
        assert_eq!(detect("file.tar.xz"), Some(ArchiveFormat::new(7, TarXz)));
        assert_eq!(detect("file.txz"), Some(ArchiveFormat::new(4, TarXz)));
        assert_eq!(
            detect("file.tar.lzma"),
            Some(ArchiveFormat::new(9, TarLzma))
        );
        assert_eq!(detect("file.tlz"), Some(ArchiveFormat::new(4, TarLzma)));
        assert_eq!(detect("file.tar.7z"), Some(ArchiveFormat::new(7, Tar7z)));
        assert_eq!(
            detect("file.tar.7z.001"),
            Some(ArchiveFormat::new(11, Tar7z))
        );
        assert_eq!(detect("file.t7z"), Some(ArchiveFormat::new(4, Tar7z)));
        assert_eq!(detect("file.tar.zst"), Some(ArchiveFormat::new(8, TarZstd)));
        assert_eq!(detect("file.tar"), Some(ArchiveFormat::new(4, Tar)));
        assert_eq!(detect("file.z"), Some(ArchiveFormat::new(2, Z)));
        assert_eq!(detect("file.zip"), Some(ArchiveFormat::new(4, Zip)));
        assert_eq!(detect("file.gz"), Some(ArchiveFormat::new(3, Gzip)));
        assert_eq!(detect("file.bz2"), Some(ArchiveFormat::new(4, Bzip2)));
        assert_eq!(detect("file.lz"), Some(ArchiveFormat::new(3, Lz)));
        assert_eq!(detect("file.xz"), Some(ArchiveFormat::new(3, Xz)));
        assert_eq!(detect("file.lzma"), Some(ArchiveFormat::new(5, Lzma)));
        assert_eq!(detect("file.7z"), Some(ArchiveFormat::new(3, P7z)));
        assert_eq!(detect("file.7z.001"), Some(ArchiveFormat::new(7, P7z)));
        assert_eq!(detect("file.rar"), Some(ArchiveFormat::new(4, Rar)));
        assert_eq!(detect("file.zst"), Some(ArchiveFormat::new(4, Zstd)));
    }

    #[test]
    fn does_not_detect_if_malformed() {
        assert_eq!(detect("filetgz"), None);
        assert_eq!(detect("filetbz2"), None);
        assert_eq!(detect("filetxz"), None);
        assert_eq!(detect("filetlz"), None);
        assert_eq!(detect("filet7z"), None);
        assert_eq!(detect("filetar"), None);
        assert_eq!(detect("filez"), None);
        assert_eq!(detect("filezip"), None);
        assert_eq!(detect("filegz"), None);
        assert_eq!(detect("filebz2"), None);
        assert_eq!(detect("filelz"), None);
        assert_eq!(detect("filexz"), None);
        assert_eq!(detect("filelzma"), None);
        assert_eq!(detect("file7z"), None);
        assert_eq!(detect("file7z.001"), None);
        assert_eq!(detect("filerar"), None);
        assert_eq!(detect("filezst"), None);
    }

    #[test]
    fn does_not_detect_tar_if_malformed() {
        assert_eq!(detect("filetar.z"), Some(ArchiveFormat::new(2, Z)));
        assert_eq!(detect("filetar.gz"), Some(ArchiveFormat::new(3, Gzip)));
        assert_eq!(detect("filetar.bz2"), Some(ArchiveFormat::new(4, Bzip2)));
        assert_eq!(detect("filetar.lz"), Some(ArchiveFormat::new(3, Lz)));
        assert_eq!(detect("filetar.xz"), Some(ArchiveFormat::new(3, Xz)));
        assert_eq!(detect("filetar.lzma"), Some(ArchiveFormat::new(5, Lzma)));
        assert_eq!(detect("filetar.7z"), Some(ArchiveFormat::new(3, P7z)));
        assert_eq!(detect("filetar.7z.001"), Some(ArchiveFormat::new(7, P7z)));
        assert_eq!(detect("filetar.zst"), Some(ArchiveFormat::new(4, Zstd)));
    }
    fn wrap_extract_path(path: &str, cache_path: &Path) -> Result<PathBuf> {
        path_for_extraction(&Path::new(path), &detect(path).expect("archive detection"), cache_path)
    }

    #[cfg(unix)]
    #[test]
    fn unix_extract_path() {
        let cache_path = Path::new("foo/bar");
        assert_eq!(
            wrap_extract_path("foo.tar.gz", cache_path).unwrap(),
            Path::new("foo/bar/foo")
        );
        assert_eq!(
            wrap_extract_path("src/foo.tar.gz", cache_path).unwrap(),
            Path::new("foo/bar/src%2Ffoo")
        );
        assert_eq!(
            wrap_extract_path("/usr/local/foo.tar.gz", cache_path).unwrap(),
            Path::new("foo/bar/usr%2Flocal%2Ffoo")
        );
        assert_eq!(
            wrap_extract_path("/etc/foo.tar.gz", cache_path).unwrap(),
            Path::new("foo/bar/etc%2Ffoo")
        );
        assert_eq!(
            wrap_extract_path("dist/front/out/foo.tar.gz", cache_path).unwrap(),
            Path::new("foo/bar/dist%2Ffront%2Fout%2Ffoo")
        );
        assert_eq!(
            wrap_extract_path("./dist/foo.tar.xz", cache_path).unwrap(),
            wrap_extract_path("dist/foo.tar.xz", cache_path).unwrap()
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_extract_path() {
        let cache_path = PathBuf::from(r"foo\bar");
        assert_eq!(
            wrap_extract_path(r"foo.tar.gz", cache_path.as_ref()).unwrap(),
            Path::new(r"foo\bar\foo")
        );
        assert_eq!(
            wrap_extract_path(r"src\foo.tar.gz", cache_path.as_ref()).unwrap(),
            Path::new(r"foo\bar\src%5Cfoo")
        );
        assert_eq!(
            wrap_extract_path(r"\Programs\Bar\foo.tar.gz", cache_path.as_ref()).unwrap(),
            Path::new(r"foo\bar\Programs%5CBar%5Cfoo")
        );
        assert_eq!(
            wrap_extract_path(r"\Projects\foo.tar.gz", cache_path.as_ref()).unwrap(),
            Path::new(r"foo\bar\Projects%5Cfoo")
        );
        assert_eq!(
            wrap_extract_path(r"dist\front\out\foo.tar.gz", cache_path.as_ref()).unwrap(),
            Path::new(r"foo\bar\dist%5Cfront%5Cout%5Cfoo")
        );
    }
}
