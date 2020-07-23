#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
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

impl ArchiveFormat {
    pub fn detect(file_name: &str) -> Option<ArchiveFormat> {
        let file_name = file_name.to_ascii_lowercase();

        if file_name.ends_with(".tar.z") {
            return Some(ArchiveFormat::TarZ);
        } else if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            return Some(ArchiveFormat::TarGzip);
        } else if file_name.ends_with(".tar.bz2") || file_name.ends_with(".tbz2") {
            return Some(ArchiveFormat::TarBzip2);
        } else if file_name.ends_with(".tar.lz") {
            return Some(ArchiveFormat::TarLz);
        } else if file_name.ends_with(".tar.xz") || file_name.ends_with(".txz") {
            return Some(ArchiveFormat::TarXz);
        } else if file_name.ends_with(".tar.lzma") || file_name.ends_with(".tlz") {
            return Some(ArchiveFormat::TarLzma);
        } else if file_name.ends_with(".tar.7z")
            || file_name.ends_with(".tar.7z.001")
            || file_name.ends_with(".t7z")
        {
            return Some(ArchiveFormat::Tar7z);
        } else if file_name.ends_with(".tar.zst") {
            return Some(ArchiveFormat::TarZstd);
        } else if file_name.ends_with(".tar") {
            return Some(ArchiveFormat::Tar);
        } else if file_name.ends_with(".z") {
            return Some(ArchiveFormat::Z);
        } else if file_name.ends_with(".zip") {
            return Some(ArchiveFormat::Zip);
        } else if file_name.ends_with(".gz") {
            return Some(ArchiveFormat::Gzip);
        } else if file_name.ends_with(".bz2") {
            return Some(ArchiveFormat::Bzip2);
        } else if file_name.ends_with(".lz") {
            return Some(ArchiveFormat::Lz);
        } else if file_name.ends_with(".xz") {
            return Some(ArchiveFormat::Xz);
        } else if file_name.ends_with(".lzma") {
            return Some(ArchiveFormat::Lzma);
        } else if file_name.ends_with(".7z") || file_name.ends_with(".7z.001") {
            return Some(ArchiveFormat::P7z);
        } else if file_name.ends_with(".rar") {
            return Some(ArchiveFormat::Rar);
        } else if file_name.ends_with(".zst") {
            return Some(ArchiveFormat::Zstd);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_correctly() {
        assert_eq!(
            ArchiveFormat::detect("file.tar.z"),
            Some(ArchiveFormat::TarZ)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.gz"),
            Some(ArchiveFormat::TarGzip)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tgz"),
            Some(ArchiveFormat::TarGzip)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.bz2"),
            Some(ArchiveFormat::TarBzip2)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tbz2"),
            Some(ArchiveFormat::TarBzip2)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.lz"),
            Some(ArchiveFormat::TarLz)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.xz"),
            Some(ArchiveFormat::TarXz)
        );
        assert_eq!(
            ArchiveFormat::detect("file.txz"),
            Some(ArchiveFormat::TarXz)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.lzma"),
            Some(ArchiveFormat::TarLzma)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tlz"),
            Some(ArchiveFormat::TarLzma)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.7z"),
            Some(ArchiveFormat::Tar7z)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.7z.001"),
            Some(ArchiveFormat::Tar7z)
        );
        assert_eq!(
            ArchiveFormat::detect("file.t7z"),
            Some(ArchiveFormat::Tar7z)
        );
        assert_eq!(
            ArchiveFormat::detect("file.tar.zst"),
            Some(ArchiveFormat::TarZstd)
        );
        assert_eq!(ArchiveFormat::detect("file.tar"), Some(ArchiveFormat::Tar));
        assert_eq!(ArchiveFormat::detect("file.z"), Some(ArchiveFormat::Z));
        assert_eq!(ArchiveFormat::detect("file.zip"), Some(ArchiveFormat::Zip));
        assert_eq!(ArchiveFormat::detect("file.gz"), Some(ArchiveFormat::Gzip));
        assert_eq!(
            ArchiveFormat::detect("file.bz2"),
            Some(ArchiveFormat::Bzip2)
        );
        assert_eq!(ArchiveFormat::detect("file.lz"), Some(ArchiveFormat::Lz));
        assert_eq!(ArchiveFormat::detect("file.xz"), Some(ArchiveFormat::Xz));
        assert_eq!(
            ArchiveFormat::detect("file.lzma"),
            Some(ArchiveFormat::Lzma)
        );
        assert_eq!(ArchiveFormat::detect("file.7z"), Some(ArchiveFormat::P7z));
        assert_eq!(
            ArchiveFormat::detect("file.7z.001"),
            Some(ArchiveFormat::P7z)
        );
        assert_eq!(ArchiveFormat::detect("file.rar"), Some(ArchiveFormat::Rar));
        assert_eq!(ArchiveFormat::detect("file.zst"), Some(ArchiveFormat::Zstd));
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
        assert_eq!(ArchiveFormat::detect("filetar.z"), Some(ArchiveFormat::Z));
        assert_eq!(
            ArchiveFormat::detect("filetar.gz"),
            Some(ArchiveFormat::Gzip)
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.bz2"),
            Some(ArchiveFormat::Bzip2)
        );
        assert_eq!(ArchiveFormat::detect("filetar.lz"), Some(ArchiveFormat::Lz));
        assert_eq!(ArchiveFormat::detect("filetar.xz"), Some(ArchiveFormat::Xz));
        assert_eq!(
            ArchiveFormat::detect("filetar.lzma"),
            Some(ArchiveFormat::Lzma)
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.7z"),
            Some(ArchiveFormat::P7z)
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.7z.001"),
            Some(ArchiveFormat::P7z)
        );
        assert_eq!(
            ArchiveFormat::detect("filetar.zst"),
            Some(ArchiveFormat::Zstd)
        );
    }
}
