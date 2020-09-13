package com.example.robusta

class Hello {
    companion object {
        init {
            println(System.getProperty("java.library.path"))
            println("${System.getProperty("user.dir")}, ${System.mapLibraryName("robusta_example")}")
            System.loadLibrary("robusta_example")
        }

        @JvmStatic
        external fun special_fuction(input1: String, input2: String): String

        @JvmStatic
        external fun hello(input: String): String
    }
}

fun main() {
    println(Hello.hello("hello"))
}