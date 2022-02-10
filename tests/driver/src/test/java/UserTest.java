import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.assertEquals;

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
        assertValueRoundTrip(u::getInt, u::intToString, 0, "0");
        assertValueRoundTrip(u::getInt, u::intToString, 1, "1");
        assertValueRoundTrip(u::getInt, u::intToString, -1, "-1");
        assertValueRoundTrip(u::getInt, u::intToString, Integer.MAX_VALUE, "2147483647");
        assertValueRoundTrip(u::getInt, u::intToString, Integer.MIN_VALUE, "-2147483648");
    }

    @Test
    public void boolTest() {
        assertValueRoundTrip(u::getBool, u::boolToString, false, "false");
        assertValueRoundTrip(u::getBool, u::boolToString, true, "true");
    }

    @Test
    public void charTest() {
        assertValueRoundTrip(u::getChar, u::charToString, 'a', "a");
        assertValueRoundTrip(u::getChar, u::charToString, '\n', "\n");
        assertValueRoundTrip(u::getChar, u::charToString, '你', "你");
        assertValueRoundTrip(u::getChar, u::charToString, Character.MIN_VALUE, "\0");
        // note: Character.MAX_VALUE != char::MAX
        assertValueRoundTrip(u::getChar, u::charToString, Character.MAX_VALUE, "\uffff");
    }

    @Test
    public void byteTest() {
        assertValueRoundTrip(u::getByte, u::byteToString, (byte) 0, "0");
        assertValueRoundTrip(u::getByte, u::byteToString, (byte) 1, "1");
        assertValueRoundTrip(u::getByte, u::byteToString, (byte) -1, "-1");
        assertValueRoundTrip(u::getByte, u::byteToString, Byte.MAX_VALUE, "127");
        assertValueRoundTrip(u::getByte, u::byteToString, Byte.MIN_VALUE, "-128");
    }

    @Test
    public void floatTest() {
        assertValueRoundTrip(u::getFloat, u::floatToString, (float) 0.0, "0");
        assertValueRoundTrip(u::getFloat, u::floatToString, (float) 1.23, "1.23");
        assertValueRoundTrip(u::getFloat, u::floatToString, (float) -123.45, "-123.45");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.MAX_VALUE, "340282350000000000000000000000000000000");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.MIN_VALUE, "0.000000000000000000000000000000000000000000001");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.NaN, "NaN");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.MIN_NORMAL, "0.000000000000000000000000000000000000011754944");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.POSITIVE_INFINITY, "inf");
        assertValueRoundTrip(u::getFloat, u::floatToString, Float.NEGATIVE_INFINITY, "-inf");
    }

    @Test
    public void doubleTest() {
        assertValueRoundTrip(u::getDouble, u::doubleToString, 0.0, "0");
        assertValueRoundTrip(u::getDouble, u::doubleToString, 1.23, "1.23");
        assertValueRoundTrip(u::getDouble, u::doubleToString, -123.45, "-123.45");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.MAX_VALUE,
                "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.MIN_VALUE,
                "0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000005");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.NaN, "NaN");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.MIN_NORMAL,
                "0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" +
                        "00000000000000000000000000000000000000022250738585072014");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.POSITIVE_INFINITY, "inf");
        assertValueRoundTrip(u::getDouble, u::doubleToString, Double.NEGATIVE_INFINITY, "-inf");
    }

    @Test
    public void longTest() {
        assertValueRoundTrip(u::getLong, u::longToString, 0L, "0");
        assertValueRoundTrip(u::getLong, u::longToString, 1L, "1");
        assertValueRoundTrip(u::getLong, u::longToString, -1L, "-1");
        assertValueRoundTrip(u::getLong, u::longToString, Long.MAX_VALUE, "9223372036854775807");
        assertValueRoundTrip(u::getLong, u::longToString, Long.MIN_VALUE, "-9223372036854775808");
    }

    @Test
    public void shortTest() {
        assertValueRoundTrip(u::getShort, u::shortToString, (short) 0, "0");
        assertValueRoundTrip(u::getShort, u::shortToString, (short) 1, "1");
        assertValueRoundTrip(u::getShort, u::shortToString, (short) -1, "-1");
        assertValueRoundTrip(u::getShort, u::shortToString, Short.MAX_VALUE, "32767");
        assertValueRoundTrip(u::getShort, u::shortToString, Short.MIN_VALUE, "-32768");
    }

    @Test
    public void stringTest() {
        assertValueRoundTrip(u::getString, Function.identity(), "", "");
        assertValueRoundTrip(u::getString, Function.identity(), "hello!", "hello!");
        assertValueRoundTrip(u::getString, Function.identity(), "a".repeat(10000), "a".repeat(10000));
        assertValueRoundTrip(u::getString, Function.identity(), "\0a\rb\nc\t", "\0a\rb\nc\t");
        assertValueRoundTrip(u::getString, Function.identity(), "아주 좋습니다", "아주 좋습니다");
        // pirate flag https://unicode.org/emoji/charts/emoji-zwj-sequences.html
        assertValueRoundTrip(u::getString, Function.identity(), "️🏴‍☠️", "️🏴‍☠️");
        assertValueRoundTrip(u::getString, Function.identity(), "️️𒅄", "️️𒅄"); // 4 bytes in utf-8
    }

    @Test
    public void stringArrayTest() {
        assertValueRoundTrip(u::getStringArray, u::stringArrayToString, List.of(), "[]");
        assertValueRoundTrip(u::getStringArray, u::stringArrayToString, List.of("a", "b", "c"), "[\"a\", \"b\", \"c\"]");
    }

    @Test
    public void staticMethod() {
        assertEquals(String.valueOf(User.getTotalUsersCount()), User.userCountStatus());
    }

    private <T> void assertValueRoundTrip(Function<T, T> func, Function<T, String> toString, T value, String text) {
        assertEquals(value, func.apply(value));
        assertEquals(text, toString.apply(value));
    }
}
