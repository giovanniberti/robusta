//! `robusta_jni` is a library that provides a procedural macro to make easier to write JNI-compatible code in Rust.
//!
//! It can perform automatic conversion of Rust-y input and output types.
//!
//! ```toml
//! [dependencies]
//! robusta_jni = "0.0.3"
//! ```
//!
//! # Getting started
//! The [`#[bridge]`](bridge) attribute is `robusta_jni`'s entry point. It must be applied to a module.
//! `robusta_jni` will then generate proper function definitions and trait implementations depending on declared methods.
//!
//! # Declaring classes
//! Rust counterparts of Java classes are declared as Rust `struct`s, with a `#[package(my.package.name)]` attribute.
//! When using the default package, just omit the package name inside parentheses.
//!
//! Structs without the package attribute will be ignored by `robusta_jni`.
//!
//! In order to use the features of `robusta_jni`, declared structs should also implement the [`Signature`] trait.
//! This can be done manually or with autoderive.
//!
//! Example:
//! ```rust
//! use robusta_jni::bridge;
//! use robusta_jni::convert::Signature;
//!
//! #[bridge]
//! mod jni {
//!     # use robusta_jni::convert::Signature;
//!     #[package()] // default package
//!     struct A;
//!
//!     impl Signature for A {
//!         const SIG_TYPE: &'static str = "LA;";
//!     }
//!
//!     #[derive(Signature)]
//!     #[package(my.awesome.package)]
//!     struct B;
//! }
//! ```
//!
//! # Adding native methods
//! JNI bindings are generated for every method implemented for `package`-annotated structs.
//! Each method can optionally specify a `#[call_type]` attribute that will determine how conversions between Rust and Java types are performed.
//! For more information about conversions and `#[call_type]`, check out the [convert](convert) module.
//!
//! In general, **all input and output types must implement proper conversion traits**
//! (input types must implement `(Try)FromJavaValue` and output types must implement `(Try)IntoJavaValue`)
//!
//! Native methods can optionally accept a [`JNIEnv`] parameter as first parameter (after `self` if present).
//!
//! Methods are declared as standard Rust functions with public visibility and "jni" ABI, and are matched by name with Java methods.
//! No special handling is needed.
//!
//! Example:
//!
//! ```rust
//! # use robusta_jni::bridge;
//! #
//! # #[bridge]
//! # mod jni {
//!     # use robusta_jni::convert::{Signature, TryFromJavaValue, JavaValue};
//!     # use robusta_jni::jni::JNIEnv;
//!     # use jni::objects::JObject;
//!     # #[derive(Signature)]
//!     # #[package()]
//!     # struct A;
//!     #
//!     # impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for A {
//!     #    type Source = JObject<'env>;
//!     #
//!     #    fn try_from(s: Self::Source,env: &'borrow JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
//!     #         Ok(A)
//!     #     }
//!     # }
//!     #
//! impl A {
//!     pub extern "jni" fn op(self, _env: &JNIEnv, flag: bool) -> i32 {
//!         //                       ^^^^^ optional
//!         if flag {
//!             1
//!         } else {
//!             0
//!         }
//!     }
//!
//!     // here the `env` parameter is omitted
//!     pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
//!         input1.push(input2);
//!         input1.iter().map(ToString::to_string).collect()
//!     }
//!
//! }
//! # }
//! ```
//!
//! # Adding Java methods
//! You can also declare Java methods and `robusta` will generate binding glue to convert types and call methods on the Java side.
//! Again, **all input and output types must implement proper conversion traits**: in this case it's the reverse from the Java🠖Rust case
//! (input types must implement `(Try)IntoJavaValue` and output types must implement `(Try)FromJavaValue`).
//!
//! Methods are declared as standard Rust functions with public visibility, a "java" ABI and an empty body, and are matched by name with Java methods.
//! Both static and non-static methods must accept a [`JNIEnv`] parameter as first parameter (after self if present).
//!
//! Constructors can be declared via a `#[constructor]` attribute on static methods, and are matched by their type signature.
//!
//! When using `#[call_type(safe)]` or omitting `call_type` attribute, the output type **must** be [`jni::errors::Result<T>`](jni::errors::Result)
//! with `T` being the actual method return type. Otherwise when using `#[call_type(unchecked)]` `T` is sufficient.
//!
//! **When using `#[call_type(unchecked)]` if a Java exception is thrown while calling a method a panic is raised.**
//!
//! ## Static methods
//!
//! Example:
//! ```rust
//! # use robusta_jni::bridge;
//! # use robusta_jni::convert::{Signature, TryFromJavaValue};
//! #
//! # #[bridge]
//! # mod jni {
//!     # use robusta_jni::convert::{Signature, TryFromJavaValue, JavaValue};
//!     # use robusta_jni::jni::JNIEnv;
//!     # use jni::objects::JObject;
//!     # #[derive(Signature)]
//!     # #[package()]
//!     # struct A;
//!     #
//!     # impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for A {
//!     #    type Source = JObject<'env>;
//!     #
//!     #    fn try_from(s: Self::Source,env: &'borrow JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
//!     #         Ok(A)
//!     #     }
//!     # }
//!     #
//! impl A {
//!     pub extern "java" fn staticJavaMethod(
//!         env: &JNIEnv,
//!         i: i32,
//!         u: i32,
//!     ) -> ::robusta_jni::jni::errors::Result<i32> {}
//! }
//! # }
//! ```
//!
//! ## Non-static methods
//!
//! Example:
//! ```rust
//! # use robusta_jni::bridge;
//! # use robusta_jni::convert::{Signature, TryFromJavaValue};
//! #
//! # #[bridge]
//! # mod jni {
//!     # use robusta_jni::convert::{Signature, TryFromJavaValue, JavaValue, TryIntoJavaValue};
//!     # use robusta_jni::jni::JNIEnv;
//!     # use jni::objects::JObject;
//!     # #[derive(Signature)]
//!     # #[package()]
//!     # struct A;
//!     #
//!     # impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for A {
//!     #    type Source = JObject<'env>;
//!     #
//!     #    fn try_from(s: Self::Source,env: &'borrow JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
//!     #         Ok(A)
//!     #     }
//!     # }
//!     #
//!     # impl<'env> TryIntoJavaValue<'env> for &A {
//!     #   type Target = JObject<'env>;
//!     #
//!     #   fn try_into(self, env: &JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self::Target> {
//!     #         env.new_object("A", "()V", &[])
//!     #   }
//!     # }
//!     #
//! impl A {
//!     pub extern "java" fn selfMethod(
//!         &self,
//!         env: &JNIEnv,
//!         i: i32,
//!         u: i32,
//!     ) -> ::robusta_jni::jni::errors::Result<i32> {}
//! }
//! # }
//! ```
//!
//! ## Constructors
//!
//! Example:
//! ```rust
//! # use robusta_jni::bridge;
//! # use robusta_jni::convert::{Signature, TryFromJavaValue};
//! #
//! # #[bridge]
//! # mod jni {
//!     # use robusta_jni::convert::{Signature, TryFromJavaValue, JavaValue};
//!     # use robusta_jni::jni::JNIEnv;
//!     # use jni::objects::JObject;
//!     # #[derive(Signature)]
//!     # #[package()]
//!     # struct A;
//!     #
//!     # impl<'env: 'borrow, 'borrow> TryFromJavaValue<'env, 'borrow> for A {
//!     #    type Source = JObject<'env>;
//!     #
//!     #    fn try_from(s: Self::Source,env: &'borrow JNIEnv<'env>) -> ::robusta_jni::jni::errors::Result<Self> {
//!     #         Ok(A)
//!     #     }
//!     # }
//!     #
//! impl A {
//!     #[constructor]  //   vvv------ this method can be anything because it's a constructor
//!     pub extern "java" fn new(
//!         env: &JNIEnv
//!     ) -> ::robusta_jni::jni::errors::Result<i32> {}
//! }
//! # }
//! ```
//!
//! # Conversion details and special lifetimes
//! The procedural macro handles two special lifetimes specially: `'env` and `'borrow`.
//!
//! When declaring structs with lifetimes you may be asked to name one of the lifetimes as `'env` in order to
//! disambiguate code generation for the attribute macro.
//! In the generated code, this lifetime would correspond to the one used to convert your type to `*IntoJavaValue`, like:
//! ```ignore
//! <A<'env> as TryIntoJavaValue<'env>>
//! ```
//! This lifetime is always used as the lifetime parameter of `JNIEnv` instances.
//!
//! When using `*FromJavaValue` derive macros your structs will be required to have both `'env` and `'borrow`,
//! with the same bounds as in the trait definition. For more information, see the relevant traits documentation.
//!
//! ## Library-provided conversions
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
//! | [jni::jobject](jni::sys::jobject)                                                    | *(any Java object as output)*     |
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
//! [`Signature`]: convert::Signature
//! [`JNIEnv`]: jni::JNIEnv
//!

pub use robusta_codegen::bridge;

pub mod convert;

pub use jni;

pub use static_assertions::assert_type_eq_all;
