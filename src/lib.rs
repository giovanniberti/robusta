//! `robusta_jni` is a library provides a procedural macro to make easier to write JNI-compatible code in Rust.
//!
//! It can perform automatic conversion of Rust-y input and output types.
//!
//! ```toml
//! [dependencies]
//! robusta_jni = "0.0.3"
//! ```
//!
//! # Getting started
//! The `#[bridge]` attribute is `robusta_jni`'s entry point. It must be applied to a module.
//! `robusta_jni` will then generate proper function definitions and trait implementation depending on declared methods.
//!
//! # Declaring classes
//! Rust counterparts of Java classes are declared as Rust `struct`s, with a `#[package(my.package.name)]` attribute.
//! When using the default package, just omit the package name inside parentheses.
//!
//! Structs without the package attribute will be ignored by `robusta_jni`.
//!
//! Example:
//! ```rust
//! use robusta_jni::bridge;
//! #[bridge]
//! mod jni {
//!     #[package()] // default package
//!     struct A;
//!
//!     #[package(my.awesome.package)]
//!     struct B;
//! }
//! ```
//!
//! # Adding native methods
//! JNI bindings are generated for every method implemented for `package`-annotated structs.
//! Each method can optionally specify a `#[call_type]` attribute that will determine how conversions between Rust and Java types is performed.
//! For more information about conversions and `#[call_type]`, check out [convert](convert) module.
//!
//! In general, **all input and output types must implement proper conversion traits**
//! (input types must implement `(Try)FromJavaValue` and output types must implement `(Try)IntoJavaValue`)
//!
//! Native methods can optionally accept a [JNIEnv](jni::JNIEnv) parameter as first parameter (after `self`).
//!
//! Methods are declared as standard Rust functions with public visibility and "jni" ABI. No special handling is needed.
//!
//! Example:
//!
//! ```ignore
//! use robusta_jni::jni::sys::JNIEnv;
//! impl A {
//!     pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
//!         input1.push(input2);
//!         input1.iter().map(ToString::to_string).collect()
//!     }
//!
//!     pub extern "jni" fn op(self, _env: JNIEnv, flag: bool) -> i32 {
//!         //                       ^^^^^ optional
//!         if flag {
//!             1
//!         } else {
//!             0
//!         }
//!     }
//! }
//! ```
//!
//! # Adding Java methods
//! You can also declare Java methods and `robusta` will generate binding glue to convert types and call methods on the Java side.
//! Again, **all input and output types must implement proper conversion traits**: in this case it's the reverse from the Java->Rust case
//! (input types must implement `(Try)IntoJavaValue` and output types must implement `(Try)FromJavaValue`).
//!
//! Methods are declared as standard Rust functions with public visibility, a "java" ABI and an empty body.
//! Depending on the method type (static or not), there are some other adjustments to be made.
//!
//! When using `#[call_type(safe)]` or omitting `call_type` attribute, the output type **must** be [jni::errors::Result\<T\>](jni::errors::Result)
//! with `T` being the actual method return type. Otherwise, if using `#[call_type(unchecked)]` `T` is sufficient.
//!
//! **When using `#[call_type(unchecked)]` if a Java exception is thrown while calling a method a panic is raised.**
//!
//! ## Static methods
//! Static methods **must** have a [JNIEnv](JNIEnv) reference as first parameter.
//!
//! Example:
//! ```ignore
//! use robusta_jni::jni::JNIEnv;
//! pub extern "java" fn staticJavaMethod(
//!             env: &JNIEnv,
//!             i: i32,
//!             u: i32,
//!         ) -> ::robusta_jni::jni::errors::Result<i32> {}
//! ```
//!
//! ## Non-static methods
//! For non-static methods the corresponding structs must implement the [JNIEnvLink](JNIEnvLink) trait.
//!
//! Example:
//! ```ignore
//! use robusta_jni::convert::JNIEnvLink;
//! use jni::JNIEnv;
//! impl JNIEnvLink for A {
//!     fn get_env<'env>(&self) -> &JNIEnv<'env> {
//!         unimplemented!()
//!     }
//! }
//!
//! impl A {
//!     pub extern "java" fn selfMethod(
//!                 &self,
//!                 i: i32,
//!                 u: i32,
//!            ) -> ::robusta_jni::jni::errors::Result<i32> {}
//! }
//! ```
//!
//! # Library-provided conversions
//!
//! | **Rust**                                                                           | **Java**                          |
//! |------------------------------------------------------------------------------------|-----------------------------------|
//! | i32                                                                                | int                               |
//! | bool                                                                               | boolean                           |
//! | char                                                                               | char                              |
//! | i8                                                                                 | byte                              |
//! | f32                                                                                | float                             |
//! | f64                                                                                | double                            |
//! | i64                                                                                | long                              |
//! | i16                                                                                | short                             |
//! | Vec\<T\>†                                                                          | ArrayList\<T\>                    |
//! | [jni::JObject<'env>](jni::objects::JObject)                                      ‡ | *(any Java object as input type)* |
//! | [jni::jobject](jni::sysjobject)                                                    | *(any Java object as output)*     |
//!
//! † Type parameter `T` must implement proper conversion types
//!
//! ‡ The special `'env` lifetime **must** be used
//!
//! ## Limitations
//!
//! Currently there are some limitations in the conversion mechanism:
//!  * Boxed types are supported only through the opaque `JObject`/`jobject` types
//!  * Automatic type conversion is limited to the table outlined above, though easily extendable if needed.
//!
//!

pub use robusta_codegen::bridge;

pub mod convert;

pub use jni;
