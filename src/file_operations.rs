use std::{env, fs, io, path};

pub fn get_persistence_file_path(filename: &str) -> String {
    let mut ids_file_path = env::var("HOME").expect("$HOME environment variable should exist");
    ids_file_path.push_str(&format!("/{filename}"));
    if !path::Path::new(&ids_file_path).exists() {
        fs::File::create(&ids_file_path).expect("Should have permission to create a file");
    }
    ids_file_path
}

pub fn get_local_ids(path: &str) -> String {
    match fs::read_to_string(&path) {
        Ok(ids) => ids,
        Err(_) => "".to_string(),
    }
}

pub fn save_local_ids(ids: Vec<String>, path: &str) -> Result<(), io::Error> {
    let ids_len = ids.len();
    if ids_len == 1 {
        fs::write(path, &ids[0])?;
    } else if ids_len > 1 {
        let ids_to_write: String = ids.join(",");
        fs::write(path, ids_to_write)?;
    }
    Ok(())
}
