import axios from "axios";
import cookies from "./cookies";

export default {
    /**
     * Get current session token.
     */
    getSessionToken() {
        return cookies.getCookie("session") || null;
    },

    /**
     * Check whether a session token is set.
     */
    hasSessionToken() {
        return (cookies.getCookie("session") || null) != null;
    },

    /**
     * Set current session token.
     */
    setSessionToken(token) {
        cookies.setCookie("session", token, 365);
    },

    /**
     * Reset current session token.
     */
    resetSessionToken() {
        cookies.setCookie("session", "", 0);
    },

    /**
     * Check whether we're authenticated.
     *
     * Validates the current session token. Returns promise with boolean.
     */
    isAuth() {
        return new Promise((resolve, reject) => {
            // Session token must be set
            if(!this.hasSessionToken()) {
                resolve(false);
                return;
            }

            // Request teams
            axios.post("/api/auth/validate", {token: this.getSessionToken()})
                .then((response) => resolve(!!response.data))
                .catch((err) => reject(err));
        });
    },
};

