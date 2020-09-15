package com.example.robusta;

import java.util.*;

class HelloWorld {
    private static native ArrayList<String> special(ArrayList<Integer> input1, int in2);

    static {
        System.loadLibrary("robusta_example");
    }

    public static void main(String[] args) {
        ArrayList<String> output = HelloWorld.special(new ArrayList<Integer>(List.of(1, 2, 3)), 4);
        System.out.println(output);
	}
}
