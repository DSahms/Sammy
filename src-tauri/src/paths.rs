use directories::ProjectDirs;
use std::path::PathBuf;

pub fn data_root() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("com", "Remanence", "Sammy") {
        dirs.data_dir().to_path_buf()
    } else {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into())).join("Sammy")
    }
}

pub fn vault_dir() -> PathBuf {
    data_root().join("vault")
}

pub fn db_path() -> PathBuf {
    vault_dir().join("soul.sqlcipher")
}

pub fn salt_path() -> PathBuf {
    vault_dir().join("salt.bin")
}

pub fn files_dir() -> PathBuf {
    vault_dir().join("files")
}

pub fn brains_config_path() -> PathBuf {
    data_root().join("brains.toml")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(vault_dir())?;
    std::fs::create_dir_all(files_dir())?;
    std::fs::create_dir_all(data_root())?;
    Ok(())
}
