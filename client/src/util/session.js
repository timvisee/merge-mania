'use strict';

import axios from "axios";

/**
 * Stores the session token.
 */
export default {
    /**
     * Get current session token.
     */
    getToken() {
        return localStorage.getItem('session') || null;
    },

    /**
     * Check whether a session token is set.
     */
    hasToken() {
        return this.getToken() != null;
    },

    /**
     * Set current session token.
     */
    setToken(token) {
        localStorage.setItem('session', token);
    },

    /**
     * Reset current session token.
     */
    resetToken() {
        localStorage.removeItem('session');
    },
};

