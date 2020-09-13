package com.example.robusta;

class HelloWorld {
    private static native String special(String input1, String input2);

    static {
        System.loadLibrary("robusta_example");
    }

    public static void main(String[] args) {
        String output = HelloWorld.special("Rust", "Java");
        System.out.println(output);
	}
}
