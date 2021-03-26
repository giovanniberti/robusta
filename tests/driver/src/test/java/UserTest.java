import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

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
    public void staticMethod() {
        assertEquals(String.valueOf(User.getTotalUsersCount()), User.userCountStatus());
    }
}
