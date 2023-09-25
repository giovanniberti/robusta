package com.example.robustaandroidexample;

import android.content.Context;
import android.util.Log;

public class RobustaAndroidExample {
    static {
        System.loadLibrary("robustaandroidexample");
    }

    public static native void runRustExample(Context context);

    public static String getAppFilesDir(Context context) {
        Log.d("ROBUSTA_ANDROID_EXAMPLE", "getAppFilesDir IN");
        return context.getFilesDir().toString();
    }
    public static int threadTest(String s) {
        Log.d("ROBUSTA_ANDROID_EXAMPLE", "threadTest IN: " + s);
        return 10;
    }

}
