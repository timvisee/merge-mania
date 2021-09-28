<template>
  <div>
    <loader v-if="loading" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading" class="text-center">
        <h1 class="h3 mb-3 fw-normal">Game</h1>

        <h5 class="h5 mb-3 fw-normal text-right float-right">Energie: 0</h5>
        <h5 class="h5 mb-3 fw-normal text-left">Geld: 1337</h5>

        <!-- Inventory grid -->
        <div class="game-grid">
            <div v-for="cell in inventory.grid.items" class="cell">
                <img v-if="cell && cell.Product" :src="'/sprites/' + cell.Product.sprite" />
                <img v-if="cell && cell.Factory" :src="'/sprites/' + cell.Factory.sprite" />
            </div>
        </div>

        <b-button-group size="lg w-100">
            <b-button type="button" variant="success" class="w-100">Samenvoegen</b-button>
            <b-button type="button" variant="info" class="w-100">Details</b-button>
        </b-button-group>

        <b-button-group size="lg w-100">
            <b-button type="button" variant="success" class="w-100">Kopen</b-button>
            <b-button type="button" variant="info" class="w-100">Verkopen</b-button>
        </b-button-group>

    </div>
  </div>
</template>

<script>
import sessionManager from "../util/session.js";

export default {
  name: "Game",
  data() {
    return {
      loading: true,
      inventory: this.$game.inventory,
    };
  },
  created() {
    // Redirect to login page if not authenticated
    this.$auth
        .isAuth()
        .then((auth) => {
            if(!auth)
                this.redirectToLogin();
        });

    this.onRouteChange();

    this.testWebsocket();
  },
  watch: {
    $route: "onRouteChange"
  },
  methods: {
    onRouteChange() {},
    redirectToLogin() {
        this.$router.push({name: "login"});
    },
    testWebsocket() {
        // Set up websocket connection
        let ws_url = window.location.origin.replace(/^http/, 'ws') + '/ws';
        let socket = new WebSocket(ws_url);

        socket.onopen = (e) => {
            console.log("[open] Connection established");
            console.log("Sending to server");
            socket.send(JSON.stringify({
                token: sessionManager.getToken(),
            }));
        };

        socket.onmessage = (event) => {
            console.log(`[message] Data received from server: ${event.data.substring(0, 32)}...`);

            // TODO: handle all incoming messages here
            let data = JSON.parse(event.data);
            this.inventory = data.data;
            this.loading = false;
        };

        socket.onclose = function(event) {
            if (event.wasClean) {
                console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
            } else {
                // e.g. server process killed or network down
                // event.code is usually 1006 in this case
                console.log('[close] Connection died');
            }
        };

        socket.onerror = function(error) {
            console.log(`[error] ${error.message}`);
        };
    },
  }
};
</script>

<style scoped>
.game-grid {
    --grid-space: 5px;
    --grid-row-cells: 8;
}

@media screen and (max-width: 560px) {
    .game-grid {
        --grid-space: 4px;
    }
}

@media screen and (max-width: 470px) {
    .game-grid {
        --grid-space: 3px;
    }
}

@media screen and (max-width: 390px) {
    .game-grid {
        --grid-space: 2px;
    }
}

@media screen and (max-width: 320px) {
    .game-grid {
        --grid-space: 1px;
    }
}

.game-grid {
    display: grid;
    margin: 2rem auto;
    padding: var(--grid-space);
    grid-template-columns: repeat(var(--grid-row-cells), 1fr);
    grid-template-rows: repeat(var(--grid-row-cells), 1fr);
    gap: var(--grid-space);
    justify-items: stretch;
    align-items: stretch;
    justify-content: stretch;
    align-content: stretch;
    aspect-ratio: 1;

    border: black solid 1px;
    box-sizing: content-box;
    background: #eee;
}

.game-grid .cell {
    display: inline-block;
    aspect-ratio: 1;
    padding: var(--grid-space);
    box-sizing: content-box;

    border: brown dashed 1px;
    text-align: center;
}

.game-grid .cell img {
    width: 100%;
    height: 100%;
}
</style>

<style>
body {
    background: #f6eada;
}

.game-grid {
    background: #dab382;
}
</style>
