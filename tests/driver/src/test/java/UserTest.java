import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.List;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;

public class UserTest {
    private User u;

    @BeforeEach
    public void setUp() {
        this.u = new User("user", "pass");
    }

    @Test
    public void selfMethod() {
        String expected = u.getPassword() + "_pass";
        String actual = u.hashedPassword(User.getTotalUsersCount());
        assertEquals(expected, actual);
    }

    @Test
    public void intTest() {
        assertValueRoundTrip(u::getInt, u::intToString, User::getIntUnchecked, User::intToStringUnchecked, 0, "0");
        assertValueRoundTrip(u::getInt, u::intToString, User::getIntUnchecked, User::intToStringUnchecked, 1, "1");
        assertValueRoundTrip(u::getInt, u::intToString, User::getIntUnchecked, User::intToStringUnchecked, -1, "-1");
        assertValueRoundTrip(u::getInt, u::intToString, User::getIntUnchecked, User::intToStringUnchecked,
                Integer.MAX_VALUE, "2147483647");
        assertValueRoundTrip(u::getInt, u::intToString, User::getIntUnchecked, User::intToStringUnchecked,
                Integer.MIN_VALUE, "-2147483648");
    }

    @Test
    public void boolTest() {
        assertValueRoundTrip(u::getBool, u::boolToString, User::getBoolUnchecked, User::boolToStringUnchecked, false,
                "false");
        assertValueRoundTrip(u::getBool, u::boolToString, User::getBoolUnchecked, User::boolToStringUnchecked, true,
                "true");
    }

    @Test
    public void charTest() {
        assertValueRoundTrip(u::getChar, u::charToString, User::getCharUnchecked, User::charToStringUnchecked, 'a',
                "a");
        assertValueRoundTrip(u::getChar, u::charToString, User::getCharUnchecked, User::charToStringUnchecked, '\n',
                "\n");
        assertValueRoundTrip(u::getChar, u::charToString, User::getCharUnchecked, User::charToStringUnchecked, '你',
                "你");
        assertValueRoundTrip(u::getChar, u::charToString, User::getCharUnchecked, User::charToStringUnchecked,
                Character.MIN_VALUE, "\0");
        // note: Character.MAX_VALUE != char::MAX
        assertValueRoundTrip(u::getChar, u::charToString, User::getCharUnchecked, User::charToStringUnchecked,
                Character.MAX_VALUE, "\uffff");
    }

    @Test
    public void byteTest() {
        assertValueRoundTrip(u::getByte, u::byteToString, User::getByteUnchecked, User::byteToStringUnchecked, (byte) 0,
                "0");
        assertValueRoundTrip(u::getByte, u::byteToString, User::getByteUnchecked, User::byteToStringUnchecked, (byte) 1,
                "1");
        assertValueRoundTrip(u::getByte, u::byteToString, User::getByteUnchecked, User::byteToStringUnchecked,
                (byte) -1, "-1");
        assertValueRoundTrip(u::getByte, u::byteToString, User::getByteUnchecked, User::byteToStringUnchecked,
                Byte.MAX_VALUE, "127");
        assertValueRoundTrip(u::getByte, u::byteToString, User::getByteUnchecked, User::byteToStringUnchecked,
                Byte.MIN_VALUE, "-128");
    }

    @Test
    public void floatTest() {
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                (float) 0.0, "0");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                (float) 1.23, "1.23");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                (float) -123.45, "-123.45");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.MAX_VALUE, "340282350000000000000000000000000000000");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.MIN_VALUE, "0.000000000000000000000000000000000000000000001");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.NaN, "NaN");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.MIN_NORMAL, "0.000000000000000000000000000000000000011754944");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.POSITIVE_INFINITY, "inf");
        assertValueRoundTrip(u::getFloat, u::floatToString, User::getFloatUnchecked, User::floatToStringUnchecked,
                Float.NEGATIVE_INFINITY, "-inf");
    }

    @Test
    public void doubleTest() {
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                0.0, "0");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                1.23, "1.23");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                -123.45, "-123.45");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.MAX_VALUE,
                "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.MIN_VALUE,
                "0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000005");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.NaN, "NaN");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.MIN_NORMAL,
                "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000022250738585072014");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.POSITIVE_INFINITY, "inf");
        assertValueRoundTrip(u::getDouble, u::doubleToString, User::getDoubleUnchecked, User::doubleToStringUnchecked,
                Double.NEGATIVE_INFINITY, "-inf");
    }

    @Test
    public void longTest() {
        assertValueRoundTrip(u::getLong, u::longToString, User::getLongUnchecked, User::longToStringUnchecked, 0L, "0");
        assertValueRoundTrip(u::getLong, u::longToString, User::getLongUnchecked, User::longToStringUnchecked, 1L, "1");
        assertValueRoundTrip(u::getLong, u::longToString, User::getLongUnchecked, User::longToStringUnchecked, -1L,
                "-1");
        assertValueRoundTrip(u::getLong, u::longToString, User::getLongUnchecked, User::longToStringUnchecked,
                Long.MAX_VALUE, "9223372036854775807");
        assertValueRoundTrip(u::getLong, u::longToString, User::getLongUnchecked, User::longToStringUnchecked,
                Long.MIN_VALUE, "-9223372036854775808");
    }

    @Test
    public void shortTest() {
        assertValueRoundTrip(u::getShort, u::shortToString, User::getShortUnchecked, User::shortToStringUnchecked,
                (short) 0, "0");
        assertValueRoundTrip(u::getShort, u::shortToString, User::getShortUnchecked, User::shortToStringUnchecked,
                (short) 1, "1");
        assertValueRoundTrip(u::getShort, u::shortToString, User::getShortUnchecked, User::shortToStringUnchecked,
                (short) -1, "-1");
        assertValueRoundTrip(u::getShort, u::shortToString, User::getShortUnchecked, User::shortToStringUnchecked,
                Short.MAX_VALUE, "32767");
        assertValueRoundTrip(u::getShort, u::shortToString, User::getShortUnchecked, User::shortToStringUnchecked,
                Short.MIN_VALUE, "-32768");
    }

    @Test
    public void stringTest() {
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(), "", "");
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(), "hello!",
                "hello!");
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(),
                "a".repeat(10000), "a".repeat(10000));
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(),
                "\0a\rb\nc\t", "\0a\rb\nc\t");
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(),
                "아주 좋습니다", "아주 좋습니다");
        // pirate flag https://unicode.org/emoji/charts/emoji-zwj-sequences.html
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(), "️🏴‍☠️",
                "️🏴‍☠️");
        assertValueRoundTrip(u::getString, Function.identity(), User::getStringUnchecked, Function.identity(), "️️𒅄",
                "️️𒅄"); // 4 bytes in utf-8
    }

    @Test
    public void intArrayTest() {
        assertValueRoundTrip(u::getIntArray, u::intArrayToString, User::getIntArrayUnchecked,
                User::intArrayToStringUnchecked, List.of(), "[]");
        assertValueRoundTrip(u::getIntArray, u::intArrayToString, User::getIntArrayUnchecked,
                User::intArrayToStringUnchecked, List.of(1, 2), "[1, 2]");
        assertValueRoundTrip(u::getIntArray, u::intArrayToString, User::getIntArrayUnchecked,
                User::intArrayToStringUnchecked, List.of(Integer.MIN_VALUE,  -1, Integer.MAX_VALUE), 
                "[-2147483648, -1, 2147483647]");
    }

    @Test
    public void stringArrayTest() {
        assertValueRoundTrip(u::getStringArray, u::stringArrayToString, User::getStringArrayUnchecked,
                User::stringArrayToStringUnchecked, List.of(), "[]");
        assertValueRoundTrip(u::getStringArray, u::stringArrayToString, User::getStringArrayUnchecked,
                User::stringArrayToStringUnchecked, List.of("a", "b", "c"), "[\"a\", \"b\", \"c\"]");
        assertArrayValueRoundTrip(u::getStringArr, u::stringArrToString, User::getStringArrUnchecked,
                User::stringArrToStringUnchecked, new String[]{}, "[]");
        assertArrayValueRoundTrip(u::getStringArr, u::stringArrToString, User::getStringArrUnchecked,
                User::stringArrToStringUnchecked, new String[]{"a", "b", "c"}, "[\"a\", \"b\", \"c\"]");
        assertArrayValueRoundTrip(u::getJStringArr, u::stringArrToString, User::getJStringArrUnchecked,
                User::stringArrToStringUnchecked, new String[]{}, "[]");
        assertArrayValueRoundTrip(u::getJStringArr, u::stringArrToString, User::getJStringArrUnchecked,
                User::stringArrToStringUnchecked, new String[]{"a", "b", "c"}, "[\"a\", \"b\", \"c\"]");
    }

    @Test
    public void optionStringArrayTest() {
        assertArrayEquals(u.getOptionStringArr(new String[]{"null", null, "42"}), new String[]{null, "null", "42"});
        assertArrayEquals(User.getOptionStringArrUnchecked(new String[]{null, "null", "42"}), new String[]{"null", null, "42"});
    }

    @Test
    public void byteArrayTest() {
        assertArrayValueRoundTrip(u::getByteArray, u::byteArrayToString, User::getByteArrayUnchecked,
                User::byteArrayToStringUnchecked, new byte[0], "[]");
        assertArrayValueRoundTrip(u::getByteArray, u::byteArrayToString, User::getByteArrayUnchecked,
                User::byteArrayToStringUnchecked, new byte[] { 1, 2, 3 }, "[1, 2, 3]");
        assertArrayValueRoundTrip(u::getByteArray, u::byteArrayToString, User::getByteArrayUnchecked,
                User::byteArrayToStringUnchecked, new byte[] { Byte.MIN_VALUE, -1, Byte.MAX_VALUE }, "[-128, -1, 127]");
    }

    @Test
    public void boolArrayTest() {
        assertArrayValueRoundTrip(u::getBoolArray, u::boolArrayToString, User::getBoolArrayUnchecked,
                User::boolArrayToStringUnchecked, new boolean[0], "[]");
        assertArrayValueRoundTrip(u::getBoolArray, u::boolArrayToString, User::getBoolArrayUnchecked,
                User::boolArrayToStringUnchecked, new boolean[] { true, false }, "[true, false]");
    }

    @Test
    public void optionTest() {
        assertValueRoundTrip(u::getOptionString, String::valueOf, User::getOptionStringUnchecked, String::valueOf, null,
                "null");
        assertValueRoundTrip(u::getOptionString, String::valueOf, User::getOptionStringUnchecked, String::valueOf, "",
                "");
        assertValueRoundTrip(u::getOptionString, String::valueOf, User::getOptionStringUnchecked, String::valueOf,
                "hello!", "hello!");
    }

    @Test
    public void staticMethod() {
        assertEquals(String.valueOf(User.getTotalUsersCount()), User.userCountStatus());
    }

    private <T> void assertValueRoundTrip(Function<T, T> func1, Function<T, String> toString1,
            Function<T, T> func2, Function<T, String> toString2, T value, String text) {
        assertEquals(value, func1.apply(value));
        assertEquals(text, toString1.apply(value));
        assertEquals(value, func2.apply(value));
        assertEquals(text, toString2.apply(value));
    }

    private void assertArrayValueRoundTrip(Function<byte[], byte[]> func1, Function<byte[], String> toString1,
            Function<byte[], byte[]> func2, Function<byte[], String> toString2, byte[] value, String text) {
        assertArrayEquals(value, func1.apply(value));
        assertEquals(text, toString1.apply(value));
        assertArrayEquals(value, func2.apply(value));
        assertEquals(text, toString2.apply(value));
    }

    private void assertArrayValueRoundTrip(Function<boolean[], boolean[]> func1, Function<boolean[], String> toString1,
            Function<boolean[], boolean[]> func2, Function<boolean[], String> toString2, boolean[] value, String text) {
        assertArrayEquals(value, func1.apply(value));
        assertEquals(text, toString1.apply(value));
        assertArrayEquals(value, func2.apply(value));
        assertEquals(text, toString2.apply(value));
    }
    
    private void assertArrayValueRoundTrip(Function<String[], String[]> func1, Function<String[], String> toString1,
            Function<String[], String[]> func2, Function<String[], String> toString2, String[] value, String text) {
        assertArrayEquals(value, func1.apply(value));
        assertEquals(text, toString1.apply(value));
        assertArrayEquals(value, func2.apply(value));
        assertEquals(text, toString2.apply(value));
    }
}
