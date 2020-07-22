use std::path::PathBuf;

#[derive(Debug)]
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

        if file_name.ends_with("tar.z") {
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
