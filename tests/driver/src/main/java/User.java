import jakarta.annotation.Nullable;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

public class User {
    static {
        System.loadLibrary("native");
        initNative();
    }

    private static int TOTAL_USERS_COUNT = 0;

    private String username;
    private String password;
    private byte[] bytes;

    @Override
    public String toString() {
        return "User{" +
                "username='" + username + '\'' +
                ", password='" + password + '\'' +
                '}';
    }

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

    public native String[] getJStringArr(String[] x);

    public static native String[] getJStringArrUnchecked(String[] x);

    public native String[] getStringArr(String[] x);

    public static native String[] getStringArrUnchecked(String[] x);

    public native @Nullable String getOptionString(@Nullable String x);

    public static native @Nullable String getOptionStringUnchecked(@Nullable String x);

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

    public native String stringArrToString(String[] x);

    public static native String stringArrToStringUnchecked(String[] x);

    public native String byteArrayToString(byte[] x);

    public static native String byteArrayToStringUnchecked(byte[] x);

    public native String boolArrayToString(boolean[] x);

    public static native String boolArrayToStringUnchecked(boolean[] x);

    private native static void initNative();

    public native static String userCountStatus();

    public native String hashedPassword(int seed);

    public User(String username) {
        this(username, username + "_pass");
    }

    public User(String username, String password) {
        User.TOTAL_USERS_COUNT += 1;

        this.username = username;
        this.password = password;
        this.bytes = password.getBytes();
    }

    public static String nullableString(@Nullable String v) {
        return v;
    }

    public static String nullableStringUnchecked(@Nullable String v) {
        return v;
    }

    public static double nullableDouble(double v) {
        return v;
    }

    public static double nullableDoubleUnchecked(double v) {
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

    public static String[][] stringArrNullable2D(
            @Nullable String[] a,
            @Nullable String[] b) {
        return new String[][] { b, a };
    }

    public String[][] stringArrNullable2DUnchecked(
            @Nullable String[] a,
            @Nullable String[] b) {
        return User.stringArrNullable2D(a, b);
    }

    // ArrayList won't work, signatures have to match
    public List<String> signaturesCheck(
            int i32,
            boolean bool,
            char character,
            byte i8,
            float f32,
            double f64,
            long i64,
            short i16,
            String string,
            List<Integer> vec_i32,
            List<String> vec_string,
            List<String> vec_option_string,
            byte[] box_i8,
            boolean[] box_bool,
            String[] box_jstring,
            String[] box_string,
            String[] box_option_string, // not passed to signaturesCheckUnchecked yet
            @Nullable String option_string,
            List<byte[]> vec_option_box_i8,
            List<byte[]> vec_box_i8,
            List<String[]> vec_option_box_string,
            List<String[]> vec_box_string,
            String[][] box_option_box_string,
            String[][] box_box_string) {
        return User.signaturesCheckUnchecked(i32, bool, character, i8, f32, f64, i64, i16, string,
                vec_i32, vec_string, vec_option_string, box_i8, box_bool, box_jstring, box_string, option_string,
                vec_option_box_i8, vec_box_i8, vec_option_box_string, vec_box_string,
                box_option_box_string, box_box_string);
    }

    public static List<String> signaturesCheckUnchecked(
            int i32,
            boolean bool,
            char character,
            byte i8,
            float f32,
            double f64,
            long i64,
            short i16,
            String string,
            List<Integer> vec_i32,
            List<String> vec_string,
            List<String> vec_option_string,
            byte[] box_i8,
            boolean[] box_bool,
            String[] box_jstring,
            String[] box_string,
            // String[] box_option_string - not possible yet,
            @Nullable String option_string,
            List<byte[]> vec_option_box_i8,
            List<byte[]> vec_box_i8,
            List<String[]> vec_option_box_string,
            List<String[]> vec_box_string,
            String[][] box_option_box_string,
            String[][] box_box_string) {
        return new ArrayList<>(List.of(
                String.valueOf(i32),
                String.valueOf(bool),
                String.valueOf(character),
                String.valueOf(i8),
                String.valueOf(f32),
                String.valueOf(f64),
                String.valueOf(i64),
                String.valueOf(i16),
                string,
                vec_i32.toString(),
                vec_string.toString(),
                vec_option_string.toString(),
                Arrays.toString(box_i8),
                Arrays.toString(box_bool),
                Arrays.toString(box_jstring),
                Arrays.toString(box_string),
                String.valueOf(option_string),
                vec_option_box_i8.stream().map(Arrays::toString).collect(Collectors.toList()).toString(),
                vec_box_i8.stream().map(Arrays::toString).collect(Collectors.toList()).toString(),
                vec_option_box_string.stream().map(Arrays::toString).collect(Collectors.toList()).toString(),
                vec_box_string.stream().map(Arrays::toString).collect(Collectors.toList()).toString(),
                Arrays.stream(box_option_box_string).map(Arrays::toString).collect(Collectors.toList()).toString(),
                Arrays.stream(box_box_string).map(Arrays::toString).collect(Collectors.toList()).toString()));
    }

    public List<String> selfSignatureCheck(
            User user,
            User borrow_user,
            @Nullable User option_borrow_user1,
            @Nullable User option_borrow_user2,
            @Nullable User option_user1,
            @Nullable User option_user2,
            List<User> vec_user,
            List<User> vec_option_user,
            @Nullable List<User> option_vec_user1,
            @Nullable List<User> option_vec_user2,
            User[] box_user,
            User[] box_option_user,
            @Nullable User[] option_box_user1,
            @Nullable User[] option_box_user2) {
        borrow_user.username += "_";
        borrow_user.password += "_";
        if (option_borrow_user1 != null) {
            option_borrow_user1.username += "_";
            option_borrow_user1.password += "_";
        }
        if (option_borrow_user2 != null) {
            option_borrow_user2.username += "_";
            option_borrow_user2.password += "_";
        }
        return List.of(
                this.toString(),
                String.valueOf(user),
                String.valueOf(option_user1),
                String.valueOf(option_user2),
                vec_user.toString(),
                vec_option_user.toString(),
                String.valueOf(option_vec_user1),
                String.valueOf(option_vec_user2),
                Arrays.toString(box_user),
                Arrays.toString(box_option_user),
                Arrays.toString(option_box_user1),
                Arrays.toString(option_box_user2));
    }

    public List<String> selfSignatureCheckUnchecked(
            User user,
            User borrow_user,
            @Nullable User option_borrow_user1,
            @Nullable User option_borrow_user2,
            @Nullable User option_user1,
            @Nullable User option_user2,
            List<User> vec_user,
            List<User> vec_option_user,
            @Nullable List<User> option_vec_user1,
            @Nullable List<User> option_vec_user2,
            User[] box_user,
            User[] box_option_user,
            @Nullable User[] option_box_user1,
            @Nullable User[] option_box_user2) {
        return selfSignatureCheck(user, borrow_user,
                option_borrow_user1, option_borrow_user2, option_user1, option_user2,
                vec_user, vec_option_user, option_vec_user1, option_vec_user2,
                box_user, box_option_user, option_box_user1, option_box_user2);
    }

    public static User cloneUser(User user) {
        return user;
    }
}
