use std::{
    path::{PathBuf, Path, Component},
    fmt,
    ffi::{OsString, OsStr},
};

use anyhow::{Result};

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
        Self { extension_length, kind }
    }
    pub fn detect(file_name: &str) -> Option<Self> {
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
        } else if  file_name.ends_with(".tbz2") {
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
}

pub fn extract(path: &str, archive: ArchiveFormat, cache_folder: &Path) -> Result<PathBuf> {
    anyhow::ensure!(archive.is_tar(), "archive format not supported: {}", archive);
    let file_name = &path[..path.len() - archive.extension_length];
    let file_name = PathBuf::from(file_name)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("there is no file_name in archive path"))?
        .to_owned();
    println!("DEBUG: file_name: {:?}", &file_name);
    let encoded_path = PathBuf::from(path);
    println!("DEBUG: encoded_path (before): {}", encoded_path.display());
    println!("DEBUG: encoded_path.parent(): {:?}", encoded_path.parent());
    let encoded_path = encoded_path.parent()
        .map(|parent| {
            if parent.as_os_str() == OsStr::new("") {
                return PathBuf::from(&file_name);
            }
            println!("DEBUG: parent: {}", parent.display());
            let mut parent = parent
                .components()
                .fold(OsString::new(), |mut p, c| {
                    match c {
                        Component::Normal(c) => {
                            p.push(c);
                            p.push("_");
                        },
                        _ => {}
                    }
                    p
                });
            parent.push(&file_name);
            PathBuf::from(parent)
        })
    .unwrap_or_else(|| PathBuf::from(&file_name));
    println!("DEBUG: encoded_path: {}", encoded_path.display());

    Ok(cache_folder.join(encoded_path))
}

#[cfg(test)]
mod test {
    use super::*;
    use ArchiveFormatKind::*;

    #[test]
    fn parses_correctly() {
        assert_eq!(
            ArchiveFormat::detect("file.tar.z"),
            Some(ArchiveFormat::new(6, TarZ))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.gz"),
            Some(ArchiveFormat::new(7, TarGzip))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tgz"),
            Some(ArchiveFormat::new(4, TarGzip))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.bz2"),
            Some(ArchiveFormat::new(8, TarBzip2))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tbz2"),
            Some(ArchiveFormat::new(5, TarBzip2))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.lz"),
            Some(ArchiveFormat::new(7, TarLz))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.xz"),
            Some(ArchiveFormat::new(7, TarXz))
        );
        assert_eq!(
            ArchiveFormat::detect("file.txz"),
            Some(ArchiveFormat::new(4, TarXz))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.lzma"),
            Some(ArchiveFormat::new(9, TarLzma))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tlz"),
            Some(ArchiveFormat::new(4, TarLzma))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.7z"),
            Some(ArchiveFormat::new(7, Tar7z))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.7z.001"),
            Some(ArchiveFormat::new(11, Tar7z))
        );
        assert_eq!(
            ArchiveFormat::detect("file.t7z"),
            Some(ArchiveFormat::new(4, Tar7z))
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.zst"),
            Some(ArchiveFormat::new(8, TarZstd))
        );
        assert_eq!(ArchiveFormat::detect("file.tar"), Some(ArchiveFormat::new(4, Tar)));
        assert_eq!(ArchiveFormat::detect("file.z"), Some(ArchiveFormat::new(2, Z)));
        assert_eq!(ArchiveFormat::detect("file.zip"), Some(ArchiveFormat::new(4, Zip)));
        assert_eq!(ArchiveFormat::detect("file.gz"), Some(ArchiveFormat::new(3, Gzip)));
        assert_eq!(
            ArchiveFormat::detect("file.bz2"),
            Some(ArchiveFormat::new(4, Bzip2))
        );
        assert_eq!(ArchiveFormat::detect("file.lz"), Some(ArchiveFormat::new(3, Lz)));
        assert_eq!(ArchiveFormat::detect("file.xz"), Some(ArchiveFormat::new(3, Xz)));
        assert_eq!(
            ArchiveFormat::detect("file.lzma"),
            Some(ArchiveFormat::new(5, Lzma))
        );
        assert_eq!(ArchiveFormat::detect("file.7z"), Some(ArchiveFormat::new(3, P7z)));
        assert_eq!(
            ArchiveFormat::detect("file.7z.001"),
            Some(ArchiveFormat::new(7, P7z))
        );
        assert_eq!(ArchiveFormat::detect("file.rar"), Some(ArchiveFormat::new(4, Rar)));
        assert_eq!(ArchiveFormat::detect("file.zst"), Some(ArchiveFormat::new(4, Zstd)));
    }

    #[test]
    fn does_not_detect_if_malformed() {
        assert_eq!(ArchiveFormat::detect("filetgz"), None);
        assert_eq!(ArchiveFormat::detect("filetbz2"), None);
        assert_eq!(ArchiveFormat::detect("filetxz"), None);
        assert_eq!(ArchiveFormat::detect("filetlz"), None);
        assert_eq!(ArchiveFormat::detect("filet7z"), None);
        assert_eq!(ArchiveFormat::detect("filetar"), None);
        assert_eq!(ArchiveFormat::detect("filez"), None);
        assert_eq!(ArchiveFormat::detect("filezip"), None);
        assert_eq!(ArchiveFormat::detect("filegz"), None);
        assert_eq!(ArchiveFormat::detect("filebz2"), None);
        assert_eq!(ArchiveFormat::detect("filelz"), None);
        assert_eq!(ArchiveFormat::detect("filexz"), None);
        assert_eq!(ArchiveFormat::detect("filelzma"), None);
        assert_eq!(ArchiveFormat::detect("file7z"), None);
        assert_eq!(ArchiveFormat::detect("file7z.001"), None);
        assert_eq!(ArchiveFormat::detect("filerar"), None);
        assert_eq!(ArchiveFormat::detect("filezst"), None);
    }

    #[test]
    fn does_not_detect_tar_if_malformed() {
        assert_eq!(ArchiveFormat::detect("filetar.z"), Some(ArchiveFormat::new(2, Z)));
        assert_eq!(
            ArchiveFormat::detect("filetar.gz"),
            Some(ArchiveFormat::new(3, Gzip))
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.bz2"),
            Some(ArchiveFormat::new(4, Bzip2))
        );
        assert_eq!(ArchiveFormat::detect("filetar.lz"), Some(ArchiveFormat::new(3, Lz)));
        assert_eq!(ArchiveFormat::detect("filetar.xz"), Some(ArchiveFormat::new(3, Xz)));
        assert_eq!(
            ArchiveFormat::detect("filetar.lzma"),
            Some(ArchiveFormat::new(5, Lzma))
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.7z"),
            Some(ArchiveFormat::new(3, P7z))
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.7z.001"),
            Some(ArchiveFormat::new(7, P7z))
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.zst"),
            Some(ArchiveFormat::new(4, Zstd))
        );
    }
    fn wrap_extract(path: &str, cache_path: &Path) -> Result<PathBuf> { extract(path, ArchiveFormat::detect(path).unwrap(), cache_path) }

    #[cfg(unix)]
    #[test]
    fn unix_extract() {
        let cache_path = PathBuf::from("foo/bar");
        assert_eq!(wrap_extract("foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new("foo/bar/foo"));
        assert_eq!(wrap_extract("src/foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new("foo/bar/src_foo"));
        assert_eq!(wrap_extract("/usr/local/foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new("foo/bar/usr_local_foo"));
        assert_eq!(wrap_extract("/etc/foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new("foo/bar/etc_foo"));
        assert_eq!(wrap_extract("dist/front/out/foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new("foo/bar/dist_front_out_foo"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_extract() {
        let cache_path = PathBuf::from(r"foo\bar");
        assert_eq!(wrap_extract(r"foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new(r"foo\bar\foo"));
        assert_eq!(wrap_extract(r"src\foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new(r"foo\bar\src_foo"));
        assert_eq!(wrap_extract(r"\Programs\Bar\foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new(r"foo\bar\Programs_Bar_foo"));
        assert_eq!(wrap_extract(r"\Projects\foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new(r"foo\bar\Projects_foo"));
        assert_eq!(wrap_extract(r"dist\front\out\foo.tar.gz", cache_path.as_ref()).unwrap(), Path::new(r"foo\bar\dist_front_out_foo"));

    }
}
