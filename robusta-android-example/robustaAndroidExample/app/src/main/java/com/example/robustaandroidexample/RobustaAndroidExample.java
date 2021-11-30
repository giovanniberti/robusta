package com.example.robustaandroidexample;

import android.content.Context;

public class RobustaAndroidExample {
    static {
        System.loadLibrary("robustaandroidexample");
    }

    public static native void runRustExample(Context context);

    public static String getAppFilesDir(Object context) {
        return ((Context) context).getFilesDir().toString();
    }

}
