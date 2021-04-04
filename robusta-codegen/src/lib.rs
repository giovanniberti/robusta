use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

use validation::JNIBridgeModule;

use crate::transformation::ModTransformer;
use derive::signature::signature_macro_derive;

mod transformation;
mod utils;
mod validation;
mod derive;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, raw_input: TokenStream) -> TokenStream {
    let module_data = parse_macro_input!(raw_input as JNIBridgeModule);

    let mut transformer = ModTransformer::new(module_data);
    let tokens = transformer.transform_module();

    tokens.into()
}

#[proc_macro_error]
#[proc_macro_derive(Signature, attributes(package))]
pub fn signature_derive(raw_input: TokenStream) -> TokenStream {
    let module_data = parse_macro_input!(raw_input as DeriveInput);

    signature_macro_derive(module_data).into()
}
