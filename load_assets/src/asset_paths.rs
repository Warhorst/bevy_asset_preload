use std::fs::read_dir;
use std::io;
use std::path::Path;

pub fn load_asset_paths() -> Vec<String> {
    load_asset_paths_recursive(Path::new("./assets")).expect("the assets folder should exist")
}

fn load_asset_paths_recursive(path: &Path) -> io::Result<Vec<String>> {
    let mut files = vec![];

    if path.is_dir() {
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(load_asset_paths_recursive(&path)?.into_iter());
            } else {
                let path_str = path
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
                    .replace("./assets/", "")
                    .to_string();
                files.push(path_str);
            }
        }
    }

    Ok(files)
}