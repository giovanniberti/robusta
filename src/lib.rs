//! This library provides a procedural macro to make easier to write JNI-compatible code in Rust.
//!
//! It can perform automatic conversion of Rust-y input and output types (see the [limitations](#limitations)).
//!
//! ```toml
//! [dependencies]
//! robusta = "0.0.1"
//! ```
//!
//! ## Usage
//! All that's needed is a couple of attributes in the right places.
//!
//! First, a `#[bridge]` attribute on a module will enable it to be processed by `robusta`.
//!
//! Then, we will need a struct for every class with a native method that will be implemented in Rust,
//! and each of these structs will have to be annotated with a `#[package]` attribute
//! with the name of the Java package the corresponding class belongs to.
//!
//! After that, the functions implemented can be written as ordinary Rust functions, and the macro will
//! take care of converting to and from Java types.
//!
//! **NOTE: This library currently supports static methods only.**
//!
//! ## Example
//! ### Rust side
//! ```rust
//! use robusta::bridge;
//!
//! #[bridge]
//! mod jni {
//!     #[package(com.example.robusta)]
//!     struct HelloWorld;
//!
//!     impl HelloWorld {
//!         fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
//!             input1.push(input2);
//!             input1.iter().map(ToString::to_string).collect()
//!         }
//!     }
//! }
//! ```
//!
//! ### Java side
//! ```java
//! package com.example.robusta;
//!
//! import java.util.*;
//!
//! class HelloWorld {
//!     private static native ArrayList<String> special(ArrayList<Integer> input1, int input2);
//!
//!     static {
//!         System.loadLibrary("robusta_example");
//!     }
//!
//!     public static void main(String[] args) {
//!         ArrayList<String> output = HelloWorld.special(new ArrayList<Integer>(List.of(1, 2, 3)), 4);
//!         System.out.println(output)
//!     }
//! }
//! ```
//!
//! ## Type conversion details and extension to custom types
//! There are two traits that control how Rust types are converted to/from Java types:
//! `FromJavaValue` and `IntoJavaValue`.
//!
//! These traits are used for input and output types respectively, and implementing them
//! is necessary to allow the library perform automatic type conversion.
//!
//! These traits make use of type provided by the  [`jni`](https://crates.io/crates/jni) crate,
//! however to provide maximum compatibility with `robusta`, we suggest using the re-exported version under `robusta::jni`.
//!
//! ### Conversion table
//!
//! | **Rust**                                                                           | **Java**                          |
//! |------------------------------------------------------------------------------------|-----------------------------------|
//! | [`i32`]                                                                            | int                               |
//! | [`bool`]                                                                           | boolean                           |
//! | [`char`]                                                                           | char                              |
//! | [`i8`]                                                                             | byte                              |
//! | [`f32`]                                                                            | float                             |
//! | [`f64`]                                                                            | double                            |
//! | [`i64`]                                                                            | long                              |
//! | [`i16`]                                                                            | short                             |
//! | [`Vec<T>`](std::vec::Vec)†                                                         | ArrayList\<T\>                    |
//! | [`JObject<'env>`](jni::JObject)‡                                                   | *(any Java object as input type)* |
//! | [`jobject`](jni::sys::jobject)                                                   | *(any Java object as output)*     |
//!
//! † Type parameter `T` must implement proper conversion types
//!
//! ‡ The special `'env` lifetime **must** be used
//!
//! ## Limitations
//! Only static methods are supported.
//!
//! Currently there are some limitations in the conversion mechanism:
//!  * Boxed types are supported only through the opaque `JObject`/`jobject` types
//!  * Automatic type conversion is limited to the table outlined above, though easily extendable if needed.
//!

pub use robusta_codegen::bridge;

pub mod convert;

#[cfg(not(feature = "no_jni"))]
pub use jni;
