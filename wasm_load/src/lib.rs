extern crate proc_macro;
use proc_macro::TokenStream;
use std::fs::read_dir;
use std::io;
use std::path::Path;

#[proc_macro]
pub fn wasm_load(_item: TokenStream) -> TokenStream {
    let files = read_files_recursive(Path::new("./assets")).expect("failed to load asset folder!");
    let load_block = files
        .into_iter()
        .map(|f| f.replace("\\", "/"))
        .map(|f| f.replace("./assets/", ""))
        .map(|f| format!("handles.push(asset_server.load_untyped(\"{f}\").untyped());"))
        .collect::<String>();

    format!("{{let mut handles = Vec::new(); {load_block} handles}}").parse().unwrap()
}

fn read_files_recursive(path: &Path) -> io::Result<Vec<String>> {
    let mut files = vec![];

    if path.is_dir() {
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(read_files_recursive(&path)?.into_iter());
            } else {
                files.push(path.to_str().unwrap().to_string());
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::read_files_recursive;

    #[test]
    fn read_files_recursive_works() {
        let result = read_files_recursive(Path::new("./assets")).unwrap();
        result.into_iter().for_each(|p| println!("{p}"))
    }
}