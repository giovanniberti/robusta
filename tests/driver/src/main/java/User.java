import java.util.ArrayList;
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

    public native List<String> getStringArray(List<String> x);

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
