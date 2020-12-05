public class User {
    static {
        System.loadLibrary("native");
        initNative();
    }

    private static int TOTAL_USERS_COUNT = 0;

    private String username;
    private String password;

    private native static int initNative();

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
