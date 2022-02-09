import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.*;

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
        assertValueRoundTrip(u::getInt, 0);
        assertValueRoundTrip(u::getInt, 1);
        assertValueRoundTrip(u::getInt, -1);
        assertValueRoundTrip(u::getInt, Integer.MAX_VALUE);
        assertValueRoundTrip(u::getInt, Integer.MIN_VALUE);
    }

    @Test
    public void boolTest() {
        assertValueRoundTrip(u::getBool, false);
        assertValueRoundTrip(u::getBool, true);
    }

    @Test
    public void charTest() {
        assertValueRoundTrip(u::getChar, 'a');
        assertValueRoundTrip(u::getChar, '\n');
        assertValueRoundTrip(u::getChar, '你');
        assertValueRoundTrip(u::getChar, Character.MIN_VALUE);
        assertValueRoundTrip(u::getChar, Character.MAX_VALUE); // note: Character.MAX_VALUE != char::MAX
    }

    @Test
    public void byteTest() {
        assertValueRoundTrip(u::getByte, (byte) 0);
        assertValueRoundTrip(u::getByte, (byte) 1);
        assertValueRoundTrip(u::getByte, (byte) -1);
        assertValueRoundTrip(u::getByte, Byte.MAX_VALUE);
        assertValueRoundTrip(u::getByte, Byte.MIN_VALUE);
    }

    @Test
    public void floatTest() {
        assertValueRoundTrip(u::getFloat, (float) 0.0);
        assertValueRoundTrip(u::getFloat, (float) 1.23);
        assertValueRoundTrip(u::getFloat, (float) -123.45);
        assertValueRoundTrip(u::getFloat, Float.MAX_VALUE);
        assertValueRoundTrip(u::getFloat, Float.MIN_VALUE);
        assertValueRoundTrip(u::getFloat, Float.NaN);
        assertValueRoundTrip(u::getFloat, Float.MIN_NORMAL);
        assertValueRoundTrip(u::getFloat, Float.POSITIVE_INFINITY);
        assertValueRoundTrip(u::getFloat, Float.NEGATIVE_INFINITY);
    }

    @Test
    public void doubleTest() {
        assertValueRoundTrip(u::getDouble, 0.0);
        assertValueRoundTrip(u::getDouble, 1.23);
        assertValueRoundTrip(u::getDouble, -123.45);
        assertValueRoundTrip(u::getDouble, Double.MAX_VALUE);
        assertValueRoundTrip(u::getDouble, Double.MIN_VALUE);
        assertValueRoundTrip(u::getDouble, Double.NaN);
        assertValueRoundTrip(u::getDouble, Double.MIN_NORMAL);
        assertValueRoundTrip(u::getDouble, Double.POSITIVE_INFINITY);
        assertValueRoundTrip(u::getDouble, Double.NEGATIVE_INFINITY);
    }

    @Test
    public void longTest() {
        assertValueRoundTrip(u::getLong, 0L);
        assertValueRoundTrip(u::getLong, 1L);
        assertValueRoundTrip(u::getLong, -1L);
        assertValueRoundTrip(u::getLong, Long.MAX_VALUE);
        assertValueRoundTrip(u::getLong, Long.MIN_VALUE);
    }

    @Test
    public void shortTest() {
        assertValueRoundTrip(u::getShort, (short) 0);
        assertValueRoundTrip(u::getShort, (short) 1);
        assertValueRoundTrip(u::getShort, (short) -1);
        assertValueRoundTrip(u::getShort, Short.MAX_VALUE);
        assertValueRoundTrip(u::getShort, Short.MIN_VALUE);
    }

    @Test
    public void stringTest() {
        assertValueRoundTrip(u::getString, "");
        assertValueRoundTrip(u::getString, "hello!");
        assertValueRoundTrip(u::getString, "a".repeat(10000));
        assertValueRoundTrip(u::getString, "\0a\rb\nc\t");
        assertValueRoundTrip(u::getString, "아주 좋습니다");
        // pirate flag https://unicode.org/emoji/charts/emoji-zwj-sequences.html
        assertValueRoundTrip(u::getString, "️🏴‍☠️");
        assertValueRoundTrip(u::getString, "️️𒅄"); // 4 bytes in utf-8
    }

    @Test
    public void stringArrayTest() {
        assertValueRoundTrip(u::getStringArray, List.of());
        assertValueRoundTrip(u::getStringArray, List.of("a", "b", "c"));
    }

    @Test
    public void staticMethod() {
        assertEquals(String.valueOf(User.getTotalUsersCount()), User.userCountStatus());
    }

    private <T> void assertValueRoundTrip(Function<T, T> func, T value) {
        assertEquals(value, func.apply(value));
    }
}
