# robusta &mdash; easy interop between Rust and Java
[![Build Status](https://github.com/giovanniberti/robusta/actions/workflows/test.yml/badge.svg)](https://github.com/giovanniberti/robusta/actions/workflows/test.yml) [![Latest Version](https://img.shields.io/crates/v/robusta_jni.svg)](https://crates.io/crates/robusta_jni) [![Docs](https://docs.rs/robusta_jni/badge.svg?version=0.2.0)](https://docs.rs/robusta_jni)

[Master branch docs](https://giovanniberti.github.io/doc/robusta_jni/)

This library provides a procedural macro to make easier to write JNI-compatible code in Rust.

It can perform automatic conversion of Rust-y input and output types (see the [limitations](#limitations)).

```toml
[dependencies]
robusta_jni = "0.2"
```

## Usage
All that's needed is a couple of attributes in the right places.

First, a `#[bridge]` attribute on a module will enable it to be processed by `robusta`.

Then, we will need a struct for every class with a native method that will be implemented in Rust,
and each of these structs will have to be annotated with a `#[package]` attribute
with the name of the Java package the corresponding class belongs to.

After that, the functions implemented can be written as ordinary Rust functions, and the macro will
take care of converting to and from Java types for functions marked public and with a `"jni"` ABI. By default if a conversion fails a Java exception is thrown.

On the other hand, if you need to call Java function from Rust, you add a `"java"` ABI and add a  `&JNIEnv` parameter after `self`/`&self`/`&mut self` (or as first parameter if the method is static), and leave the function body empty.

On these methods you can attach a `call_type` attribute that manages how conversions and errors are handled: by default, `#[call_type(safe)]` is implied,
but you can switch to `#[call_type(unchecked)]` at any time, most likely with few or no code changes.

You can also force a Java type on input arguments via `#[input_type]` attribute, which can be useful for Android JNI development for example.

### Android specificities

On Android App, to call a Java class from rust the JVM use the callstack to find desired class.
But when in a rust thread, you don't have a call stack anymore.\
So to be able to call a Java class you have to pass the class reference rather than the string class path.

You can find an example of this usage in `robusta-android-example/src/thread_func.rs`

## Code example

You can find an example under `./robusta-example`. To run it you should have `java` and `javac` on your PATH and then execute:

```bash
$ cd robusta-example
$ make java_run

# if you don't have `make` installed:
$ cargo build && javac com/example/robusta/HelloWorld.java && RUST_BACKTRACE=full java -Djava.library.path=../target/debug com.example.robusta.HelloWorld
```

### Usage on Android example

You can find an example of Robusta used for Android in `./robusta-android-example`.
To run it, open the project robustaAndroidExample with Android Studio.

Cargo build is automatically run by gradle.

The rust lib.rs is the image of the Java class RobustaAndroidExample.

This example only gets the files authorized path of the App.

## Example usage
### Rust side
```rust
use robusta_jni::bridge;
use robusta_jni::convert::Signature;

#[bridge]
mod jni {
    #[derive(Signature)]
    #[package(com.example.robusta)]
    struct HelloWorld;

    impl HelloWorld {
        pub extern "jni" fn special(mut input1: Vec<i32>, input2: i32) -> Vec<String> {
            input1.push(input2);
            input1.iter().map(ToString::to_string).collect()
        }
    }
}
```

### Java side
```java
package com.example.robusta;

import java.util.*;

class HelloWorld {
    private static native ArrayList<String> special(ArrayList<Integer> input1, int input2);

    static {
        System.loadLibrary("robusta_example");
    }

    public static void main(String[] args) {
        ArrayList<String> output = HelloWorld.special(new ArrayList<Integer>(List.of(1, 2, 3)), 4);
        System.out.println(output)
    }
}
```

## Type conversion details and extension to custom types
There are four traits that control how Rust types are converted to/from Java types:
`(Try)FromJavaValue` and `(Try)IntoJavaValue`.

These traits are used for input and output types respectively, and implementing them
is necessary to allow the library to perform automatic type conversion.

These traits make use of type provided by the  [`jni`](https://crates.io/crates/jni) crate,
however to provide maximum compatibility with `robusta`, we suggest using the re-exported version under `robusta_jni::jni`.

### Raising exceptions
You can make a Rust native method raise a Java exception simply by returning a `jni::errors::Result` with an `Err` variant.

### Conversion table

| **Rust**                                                                           | **Java**                          |
|------------------------------------------------------------------------------------|-----------------------------------|
| i32                                                                                | int                               |
| bool                                                                               | boolean                           |
| char                                                                               | char                              |
| i8                                                                                 | byte                              |
| f32                                                                                | float                             |
| f64                                                                                | double                            |
| i64                                                                                | long                              |
| i16                                                                                | short                             |
| String                                                                             | String                            |
| Vec\<T\>†                                                                          | ArrayList\<T\>                    |
| Box<[u8]>                                                                          | byte[]                            |
| Box<[bool]>                                                                        | boolean[]                         |
| [jni::JObject<'env>](https://docs.rs/jni/0.17.0/jni/objects/struct.JObject.html) ‡ | *(any Java object as input type)* |
| [jni::jobject](https://docs.rs/jni/0.17.0/jni/sys/type.jobject.html)               | *(any Java object as output)*     |

† Type parameter `T` must implement proper conversion types

‡ The special `'env` lifetime **must** be used

## Limitations

Currently there are some limitations in the conversion mechanism:
 * Boxed types are supported only through the opaque `JObject`/`jobject` types
 * Automatic type conversion is limited to the table outlined above, though easily extendable if needed.


## Contributing
I glady accept external contributions! :)
