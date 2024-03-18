use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

use validation::JNIBridgeModule;

use crate::derive::convert::{
    from_java_value_macro_derive, into_java_value_macro_derive, tryfrom_java_value_macro_derive,
    tryinto_java_value_macro_derive,
};
use crate::transformation::ModTransformer;
use derive::signature::signature_macro_derive;
use derive::arr_signature::arr_signature_macro_derive;

mod derive;
mod transformation;
mod utils;
mod validation;

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
    let input = parse_macro_input!(raw_input as DeriveInput);

    signature_macro_derive(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(ArrSignature, attributes(package))]
pub fn arr_signature_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    arr_signature_macro_derive(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(IntoJavaValue, attributes(package, instance, field))]
pub fn into_java_value_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    into_java_value_macro_derive(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(TryIntoJavaValue, attributes(package, instance, field))]
pub fn tryinto_java_value_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    tryinto_java_value_macro_derive(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(FromJavaValue, attributes(package, instance, field))]
pub fn from_java_value_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    from_java_value_macro_derive(input).into()
}

#[proc_macro_error]
#[proc_macro_derive(TryFromJavaValue, attributes(package, instance, field))]
pub fn tryfrom_java_value_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    tryfrom_java_value_macro_derive(input).into()
}
