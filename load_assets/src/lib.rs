extern crate proc_macro;

use proc_macro::TokenStream;

use crate::asset_paths::load_asset_paths;

mod asset_paths;

/// Generates an expression with collects all asset paths in a vector and returns it.
#[proc_macro]
pub fn load_assets(_item: TokenStream) -> TokenStream {
    let paths = load_asset_paths();
    let mut expression = "{let mut paths = Vec::new();".to_string();

    for path in paths {
        expression += format!("paths.push(\"{path}\");").as_str();
    }

    expression += "paths}";

    expression.parse().unwrap()
}