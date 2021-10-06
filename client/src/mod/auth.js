import axios from "axios";
import sessionManager from "../util/session";

export default {
    // Authentication state cache, null if unknown.
    auth: null,

    // Session state from server.
    session: null,

    // Attempt to authenticate with given form data.
    login(formdata) {
        return new Promise((resolve, reject) => {
            // Attempt to login
            axios.post("/api/auth/login", formdata)
                .then((response) => {
                    let token = response.data.token;
                    sessionManager.setToken(token);
                    this.auth = true;
                    resolve();
                })
                .catch((error) => {
                    // TODO: improve error handling
                    let msg = error.response.data.message;
                    console.log("Error: " + msg);
                    reject(msg);
                });
        });
    },

    /**
     * Logout.
     */
    logout() {
        // TODO: invalidate current session on server
        sessionManager.resetToken();
        this.auth = true;
    },

    /**
     * Check whether we're authenticated.
     *
     * Caches authentication state.
     */
    isAuth() {
        return new Promise((resolve, reject) => {
            if(this.auth !== null)
                resolve(this.auth);
            else
                this.checkAuth().then(resolve, reject);
        });
    },

    /**
     * Check whether we're authenticated.
     *
     * Validates the current session token. Returns promise with boolean.
     *
     * Always validates on the server.
     */
    checkAuth() {
        return new Promise((_resolve, reject) => {
            // Rewrite resolve to set auth state in here
            let resolve = (auth) => {
                this.auth = auth;
                _resolve(auth);
            };

            // Session token must be set
            if(!sessionManager.hasToken()) {
                resolve(false);
                return;
            }

            // Validate session token
            axios.post("/api/auth/validate", {token: sessionManager.getToken()})
                .then((response) => resolve(!!response.data))
                .catch((err) => reject(err));
        });
    },
};

