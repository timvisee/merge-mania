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
                    let session = response.data.session;

                    sessionManager.setToken(token.token);
                    this._setSession(session);

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
            let resolve = (session) => {
                this._setSession(session);
                _resolve(session);
            };

            // Session token must be set
            if(!sessionManager.hasToken()) {
                resolve(false);
                return;
            }

            // Validate session token
            axios.post("/api/auth/validate", {token: sessionManager.getToken()})
                .then((response) => resolve(response.data))
                .catch((err) => reject(err));
        });
    },

    /**
     * Check whether the user has the game role.
     */
    hasRoleGame() {
        return this.session != null && this.session.role_game;
    },

    /**
     * Check whether the user has the admin role.
     */
    hasRoleAdmin() {
        return this.session != null && this.session.role_admin;
    },

    /**
     * Set session.
     */
    _setSession(session) {
        // Clear session state if invalid
        if(session == null) {
            this.auth = null;
            this.session = null;
            return;
        }

        // Set session
        this.session = session;
        this.auth = session != null && session.user_id;
    },
};

