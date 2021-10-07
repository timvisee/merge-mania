import sessionManager from "../util/session.js";

export default {
    // Socket.
    socket: null,

    // App state.
    app: null,

    // Map of message listener callbacks by kind.
    listeners: {},

    // Messages queued to be sent as soon as possible.
    sendQueue: [],

    /**
     * Start new connection.
     */
    connect(app) {
        // Clean up any existing socket
        if(this.socket !== null)
            this.socket.close();

        this.app = app;
        this.app.ready = false;

        // Register internal message listeners
        this.addListener('session', (data) => this.onSession(data));
        this.addListener('toast', (data) => this.onToast(data));

        // Set up websocket connection and handlers
        this.socket = new WebSocket(socketAddress());
        this.socket.onopen = (e) => this.onOpen(e);
        this.socket.onmessage = (e) => this.onMessage(e);
        this.socket.onerror = (e) => this.onError(e);
        this.socket.onclose = (e) => this.onClose(e);
    },

    /**
     * Reconnect.
     */
    reconnect() {
        console.log("[ws] Reconnecting...");
        this.connect(this.app)
    },

    /**
     * Register a new message listener.
     */
    addListener(kind, callback) {
        // TODO: remove after debugging
        console.debug("[ws] Registered '" + kind + "' listener");
        this.listeners[kind] = callback;
    },

    /**
     * Send a message over the socket.
     */
    send(kind, data) {
        // Socket must be active
        if(this.socket == null) {
            console.log('[ws] Failed to send message, socket is null');
            return;
        }

        // Build message to send
        let msg = {
            status: 'ok',
            kind,
            data,
        };

        // Queue message if socket is not yet open
        if(this.socket.readyState != 1) {
            console.log("[ws] Socket not yet open, queueing message...");
            this.sendQueue.push(msg);
            return;
        }

        // Send message
        this.socket.send(JSON.stringify(msg));
    },

    /**
     * Invoked when websocket connection is opened.
     */
    onOpen(event) {
        console.log("[ws] Connection established");

        // TODO: send auth state to server
        this.socket.send(JSON.stringify({
            token: sessionManager.getToken(),
        }));

        // Process send queue
        this.sendQueue.forEach((msg) => this.socket.send(JSON.stringify(msg)));
        this.sendQueue = [];

        // Force refresh game/config/inventory state for any missed events
        // TODO: this is a hack, listen for reconnects in game.js itself
        if(this.app.game != null)
            this.app.game.pollGameState();
    },

    /**
     * Invoked when a message is received over the websocket.
     */
    onMessage(event) {
        // console.log(`[ws] Received msg: ${event.data.substring(0, 32)}...`);

        let data = JSON.parse(event.data);

        let listener = this.listeners[data.kind];
        if(listener != undefined)
            listener(data.data);

        // Warning for unhandled messages
        else
            console.warn("[ws] Unhandled server message: " + data.kind);
    },

    /**
     * Invoked on websocket error.
     */
    onError(error) {
        console.log(`[ws] Error: ${error.message}`);
    },

    /**
     * Invoked on websocket close.
     */
    onClose(event) {
        if (event.wasClean) {
            console.log(`[ws] Connection closed (code: ${event.code}, reason: ${event.reason})`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            console.log('[ws] Connection died');
        }

        // Reset socket and game ready state
        this.socket = null;
        this.app.ready = false;

        // Auto reconnect after some time
        console.log("[ws] Reconnecting after 2 seconds...");
        setTimeout(() => this.reconnect(), 2000);
    },

    /**
     * Handle session message.
     */
    onSession(session) {
        // TODO: using vueContext here is a hack, improve
        this.app.vueContext.$auth.session = session;
        this.app.ready = true;
    },

    /**
     * Handle toast message.
     */
    onToast(msg) {
        this.app.toast(msg);
    },
};

/**
 * Get the socket address.
 */
function socketAddress() {
    return window.location.origin.replace(/^http/, 'ws') + '/ws';
}
