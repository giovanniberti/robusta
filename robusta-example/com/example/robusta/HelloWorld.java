package com.example.robusta;

import java.util.*;

class HelloWorld {
    private static native ArrayList<String> special(ArrayList<Integer> input1, int in2);

    // pub extern "java" fn staticJavaAdd(i: i32, u: i32) -> i32 {}
    public static int staticJavaAdd(int i, int u) {
        return i + u;
    }

    // pub extern "java" fn javaAdd(&self, i: i32, u: i32) -> i32 {}
    public int javaAdd(int i, int u) {
        return i + u;
    }

    public String javaAdd(String i, int f, String u) {
            return i + u;
    }

    // pub extern "jni" fn nativeFun(self, static_call: bool) -> i32
    public native int nativeFun(boolean staticCall);

    static {
        System.loadLibrary("robusta_example");
    }

    public static void main(String[] args) {
        ArrayList<String> output = HelloWorld.special(new ArrayList<Integer>(List.of(1, 2, 3)), 4);
        System.out.println(output);

        HelloWorld h = new HelloWorld();
        System.out.println(h.nativeFun(false));
        System.out.println(h.nativeFun(true));
	}
}
