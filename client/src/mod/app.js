import ws from "./ws.js";
import game from "./game.js";

// Global app controller.
//
// Connects to server after authentication.
export default {
    // App readyness state.
    //
    // Changes to true when app is connected to the server.
    ready: false,

    // Vue context.
    // TODO: can we remove this all together?
    vueContext: null,

    // Socket manager.
    socket: null,

    // Game state, if available.
    game: null,

    // Initialize.
    init(vueContext) {
        // Init only once
        if(this.vueContext !== null) {
            console.warn('[app] Already initialized, skipping...');
            return;
        }

        console.log('[app] Initializing...');

        this.vueContext = vueContext;

        // Init socket
        this.socket = ws;
        this.socket.connect(this);
    },

    // Initialize game.
    initGame() {
        // Init game only once
        if(this.game !== null) {
            // TODO: move this into game module itself
            console.warn('[game] Already initialized, skipping...');
            return;
        }

        // Init game
        this.game = game;
        this.game.init(this);
    },

    // Show toast notification.
    toast(msg) {
        this.vueContext.$bvToast.toast(msg, {
            title: 'Notification',
            autoHideDelay: 3000,
            variant: 'warning',
            solid: true,
            appendToast: false,
        })
    },
};
