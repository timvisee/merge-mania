'use strict';

export default {
    /**
     * Get a cookie by name.
     *
     * @param string name Cookie name.
     * @return string|null Cookie value.
     */
    getCookie(name) {
        var cookies, c;
        cookies = document.cookie.split(';');
        for (var i=0; i < cookies.length; i++) {
            c = cookies[i].split('=');
            if (c[0] == name) {
                return c[1];
            }
        }
        return null;
    },

    /**
     * Get a cookie by name.
     *
     * @param string name Cookie name.
     * @param string value Cookie value.
     * @param int expiry_days Number of days to expire after.
     */
    setCookie(name, value, expiry_days) {
        var d, expires;
        expiry_days = expiry_days || 1;
        d = new Date();
        d.setTime(d.getTime() + (expiry_days * 24 * 60 * 60 * 1000));
        expires = "expires=" + d.toUTCString();
        document.cookie = name + "=" + value + "; " + expires;
    },
};

