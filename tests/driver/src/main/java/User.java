import java.util.List;

public class User {
    static {
        System.loadLibrary("native");
        initNative();
    }

    private static int TOTAL_USERS_COUNT = 0;

    private String username;
    private String password;

    public native int getInt(int x);

    public static native int getIntUnchecked(int x);

    public native boolean getBool(boolean x);

    public static native boolean getBoolUnchecked(boolean x);

    public native char getChar(char x);

    public static native char getCharUnchecked(char x);

    public native byte getByte(byte x);

    public static native byte getByteUnchecked(byte x);

    public native float getFloat(float x);

    public static native float getFloatUnchecked(float x);

    public native double getDouble(double x);

    public static native double getDoubleUnchecked(double x);

    public native long getLong(long x);

    public static native long getLongUnchecked(long x);

    public native short getShort(short x);

    public static native short getShortUnchecked(short x);

    public native String getString(String x);

    public static native String getStringUnchecked(String x);

    public native List<Integer> getIntArray(List<Integer> x);

    public static native List<Integer> getIntArrayUnchecked(List<Integer> x);

    public native List<String> getStringArray(List<String> x);

    public static native List<String> getStringArrayUnchecked(List<String> x);

    public native byte[] getByteArray(byte[] x);

    public static native byte[] getByteArrayUnchecked(byte[] x);

    public native boolean[] getBoolArray(boolean[] x);

    public static native boolean[] getBoolArrayUnchecked(boolean[] x);

    public native String getOptionString(String x);

    public static native String getOptionStringUnchecked(String x);

    public native String intToString(int x);

    public static native String intToStringUnchecked(int x);

    public native String boolToString(boolean x);

    public static native String boolToStringUnchecked(boolean x);

    public native String charToString(char x);

    public static native String charToStringUnchecked(char x);

    public native String byteToString(byte x);

    public static native String byteToStringUnchecked(byte x);

    public native String floatToString(float x);

    public static native String floatToStringUnchecked(float x);

    public native String doubleToString(double x);

    public static native String doubleToStringUnchecked(double x);

    public native String longToString(long x);

    public static native String longToStringUnchecked(long x);

    public native String shortToString(short x);

    public static native String shortToStringUnchecked(short x);

    public native String intArrayToString(List<Integer> x);

    public static native String intArrayToStringUnchecked(List<Integer> x);

    public native String stringArrayToString(List<String> x);

    public static native String stringArrayToStringUnchecked(List<String> x);

    public native String byteArrayToString(byte[] x);

    public static native String byteArrayToStringUnchecked(byte[] x);

    public native String boolArrayToString(boolean[] x);

    public static native String boolArrayToStringUnchecked(boolean[] x);

    private native static void initNative();

    public native static String userCountStatus();

    public native String hashedPassword(int seed);

    public User(String username, String password) {
        User.TOTAL_USERS_COUNT += 1;

        this.username = username;
        this.password = password;
    }

    public static String getNullableString(String v) {
        return v;
    }

    public static String getNullableStringUnchecked(String v) {
        return v;
    }

    public static int getTotalUsersCount() {
        return TOTAL_USERS_COUNT;
    }

    public static int getTotalUsersCountUnchecked() {
        return TOTAL_USERS_COUNT;
    }

    public String getPassword() {
        return password;
    }

    public String getPasswordUnchecked() {
        return password;
    }

    public String multipleParameters(int i, String s) {
        return s;
    }

    public String multipleParametersUnchecked(int i, String s) {
        return s;
    }
}
