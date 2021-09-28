import axios from "axios";
import sessionManager from "../util/session";

export default {
    // Authentication state cache, null if unknown.
    auth: null,

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
                    console.log("Error: " + error.response.data.message);
                    reject(error.response.data.message);
                });
        });
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

