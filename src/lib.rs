mod validation;
mod transformation;

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

use validation::JNIBridgeModule;
use crate::transformation::ModTransformer;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, raw_input: TokenStream) -> TokenStream {
    let module_data = parse_macro_input!(raw_input as JNIBridgeModule);
    println!("Package map: {:?}", module_data.package_map);

    let mut transformer = ModTransformer::new(module_data);
    let tokens = transformer.transform_module();

    tokens.into()
}
