import ws from "./ws.js";

export default {
    // Game readyness state.
    //
    // Changes to true when game state is received from server.
    ready: false,

    // Current inventory.
    inventory: null,

    // Game configuration items.
    items: null,

    // Game socket manager.
    socket: null,

    // Initialize.
    init() {
        this.ready = false;

        // TODO: construct fresh socket!
        this.socket = ws;
        this.socket.connect(this);
    },
};
