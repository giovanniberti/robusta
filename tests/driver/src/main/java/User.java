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

    public native boolean getBool(boolean x);

    public native char getChar(char x);

    public native byte getByte(byte x);

    public native float getFloat(float x);

    public native double getDouble(double x);

    public native long getLong(long x);

    public native short getShort(short x);

    public native String getString(String x);

    public native List<Integer> getIntArray(List<Integer> x);

    public native List<String> getStringArray(List<String> x);

    public native String intToString(int x);

    public native String boolToString(boolean x);

    public native String charToString(char x);

    public native String byteToString(byte x);

    public native String floatToString(float x);

    public native String doubleToString(double x);

    public native String longToString(long x);

    public native String shortToString(short x);

    public native String intArrayToString(List<Integer> x);

    public native String stringArrayToString(List<String> x);

    private native static void initNative();

    public native static String userCountStatus();

    public native String hashedPassword(int seed);

    public User(String username, String password) {
        User.TOTAL_USERS_COUNT += 1;

        this.username = username;
        this.password = password;
    }

    public static int getTotalUsersCount() {
        return TOTAL_USERS_COUNT;
    }

    public String getPassword() {
        return password;
    }
}
