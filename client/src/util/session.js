import axios from "axios";
import cookies from "./cookies";

/**
 * Stores the session token.
 */
export default {
    /**
     * Get current session token.
     */
    getToken() {
        return cookies.getCookie("session") || null;
    },

    /**
     * Check whether a session token is set.
     */
    hasToken() {
        return (cookies.getCookie("session") || null) != null;
    },

    /**
     * Set current session token.
     */
    setToken(token) {
        cookies.setCookie("session", token, 365);
    },

    /**
     * Reset current session token.
     */
    resetToken() {
        cookies.setCookie("session", "", 0);
    },
};

