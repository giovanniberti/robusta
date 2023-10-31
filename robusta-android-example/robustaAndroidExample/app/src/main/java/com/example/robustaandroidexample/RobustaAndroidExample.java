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
    public static int threadTestNoClass(String s) {
        Log.d("ROBUSTA_ANDROID_EXAMPLE", "threadTestNoClass IN: " + s);
        return 10;
    }
    public static int threadTestWithClass(String s) {
        Log.d("ROBUSTA_ANDROID_EXAMPLE", "threadTestWithClass IN: " + s);
        return 10;
    }

}
