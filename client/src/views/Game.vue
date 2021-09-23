<template>
  <div>
    <loader v-if="loading" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div class="text-center">
        <h1 class="h3 mb-3 fw-normal">Game</h1>

        <!-- Inventory grid -->
        <div class="game-grid">
            <div v-for="row in inventory" class="row">
                <div v-for="cell in row" class="game-cell">
                    <!-- {{ cell }} -->
                    <!-- <img src="/sprites/red-apple.png" /> -->
                </div>
            </div>
        </div>

    </div>
  </div>
</template>

<script>
import auth from "../auth";

export default {
  name: "Game",
  data() {
    return {
      loading: true,
      inventory: [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
      ],
    };
  },
  created() {
    // Redirect to login page if not authenticated
    auth.isAuth()
        .then((auth) => {
            if(!auth)
                this.redirectToLogin();
        });

    this.fetchData();

    this.testWebsocket();
  },
  watch: {
    $route: "fetchData"
  },
  methods: {
    fetchData() {
      this.loading = false;
    },

    redirectToLogin() {
        this.$router.push({name: "login"});
    },
    testWebsocket() {
        let ws_url = window.location.origin.replace(/^http/, 'ws') + '/ws';

        // let socket = new WebSocket("ws://");
        let socket = new WebSocket(ws_url);

        // TODO: remove console.log here

        socket.onopen = function(e) {
            console.log("[open] Connection established");
            console.log("Sending to server");
            socket.send(JSON.stringify({
                token: auth.getSessionToken(),
            }));
        };

        socket.onmessage = function(event) {
            console.log(`[message] Data received from server: ${event.data}`);
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
    --grid-cell-size: 48px;
    --grid-space: 5px;
}

@media screen and (max-width: 560px) {
    .game-grid {
        --grid-cell-size: 40px;
        --grid-space: 4px;
    }
}

@media screen and (max-width: 470px) {
    .game-grid {
        --grid-cell-size: 32px;
        --grid-space: 3px;
    }
}

@media screen and (max-width: 390px) {
    .game-grid {
        --grid-cell-size: 28px;
        --grid-space: 2px;
    }
}

@media screen and (max-width: 320px) {
    .game-grid {
        --grid-cell-size: 24px;
        --grid-space: 1px;
    }
}

.game-grid {
    width: fit-content;
    min-width: max-content;
    max-width: fit-content;
    display: block;
    border: black solid 1px;
    margin: 2rem auto;
    box-sizing: content-box;
    padding: var(--grid-space) 0 0 var(--grid-space);
    background: #eee;
}

.game-grid .row {
    /* display: block; */
    margin: 0 0 var(--grid-space) 0;
    box-sizing: content-box;
}

.game-grid .game-cell {
    border: brown dashed 1px;
    width: var(--grid-cell-size);
    height: var(--grid-cell-size);
    display: inline-block;
    margin: 0 var(--grid-space) 0 0;
    padding: var(--grid-space);
    box-sizing: content-box;
    text-align: center;

    background-image: url(/sprites/red-apple.png);
    background-clip: content-box;
    background-origin: content-box;
    background-repeat: no-repeat;
    background-position: center;
    background-size: contain;
}
</style>
