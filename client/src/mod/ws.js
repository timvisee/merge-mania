import sessionManager from "../util/session.js";

export default {
    // Socket.
    socket: null,

    // Game state.
    game: null,

    // Vue context.
    vueContext: null,

    /**
     * Start new connection.
     */
    connect(gameState) {
        // Clean up any existing socket
        if(this.socket !== null)
            this.socket.close();

        this.game = gameState;

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
        this.game.ready = false;
        this.connect(this.game)
    },

    /**
     * Send a message over the socket.
     */
    send(kind, data) {
        // Socket must be active
        if(this.socket === null) {
            console.log('[ws] Failed to send message, socket is null');
            return;
        }

        // Send message
        this.socket.send(JSON.stringify({
            status: 'ok',
            kind,
            data,
        }));
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
    },

    /**
     * Invoked when a message is received over the websocket.
     */
    onMessage(event) {
        // console.log(`[ws] Received msg: ${event.data.substring(0, 32)}...`);

        // TODO: handle all incoming messages here
        let data = JSON.parse(event.data);

        switch(data.kind) {
            case 'inventory':
                this.game.inventory = data.data;
                this.game.ready = true;
                break;

            case 'inventory_balances':
                this.game.inventory.money = data.data.money;
                this.game.inventory.energy = data.data.energy;
                break;

            case 'inventory_cell':
                let index = data.data.index;
                this.game.setCell(index, data.data.item);
                break;

            case 'config_items':
                this.game.items = data.data;
                break;

            case 'toast':
                this.vueContext.$bvToast.toast(data.data, {
                    title: 'Notification',
                    autoHideDelay: 3000,
                    variant: 'warning',
                    solid: true,
                    appendToast: false,
                })
                break;

            default:
                console.log("[ws] Unhandled message kind: " + data.kind);
        }
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
        this.game.ready = false;

        // Auto reconnect after some time
        console.log("[ws] Reconnecting after 2 seconds...");
        setTimeout(() => this.reconnect(), 2000);
    },
};

/**
 * Get the socket address.
 */
function socketAddress() {
    return window.location.origin.replace(/^http/, 'ws') + '/ws';
}
