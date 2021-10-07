<template>
  <div>
    <loader v-if="!app.ready || !app.game || !app.game.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">Admin</h1>

        <table class="simple-table">
            <tr>
                <td>Game:</td>
                <td>
                    <span v-if="app.running">Running</span>
                    <span v-else>Paused</span>
                </td>
            </tr>
        </table>

        <b-button
            type="button"
            size="lg"
            :variant="app.running ? 'danger' : 'success'"
            class="w-100 mt-4"
            @click.prevent.stop="playPause"
        >
            <span v-if="app.running">Pause game</span>
            <span v-else>Play game</span>
        </b-button>

        <b-button
            type="button"
            size="lg"
            variant="outline-danger"
            class="w-100 mt-4"
            @click.prevent.stop="reset"
        >
            Reset game
        </b-button>

    </div>
  </div>
</template>

<script>
import axios from "axios";

export default {
  name: "Stats",
  data() {
    return {
      app: this.$app,
    };
  },
  created() {
    // Check auth, initialize game or redirect to login
    this.$auth
        .isAuth()
        .then((auth) => {
            // TODO: require admin!

            if(auth) {
                this.$app.init(this);
                this.$app.initGame();
            } else {
                this.redirectToLogin();
                return;
            }
        });
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    playPause() {
        console.debug("[admin] Play/pause game");

        // Send play/pause command
        this.app.socket.send('set_game_running', !this.app.running);
    },

    reset() {
        console.debug("[admin] Reset game");

        // TODO: implement this

        alert('not yet implemented');
    },
  },
};
</script>

<style scoped>
.page-small-card {
    width: 100%;
    max-width: 330px;
    padding: 15px;
    margin: auto;
    margin-top: auto;
}

.simple-table {
    width: 100%;
}

.simple-table tr {
    border-bottom: 1px solid lightgray;
}

.simple-table tr:first-child {
    border-top: 1px solid lightgray;
}

.simple-table tr td {
    width: 50%;
    padding: 0.2em 0.5em;
    text-align: left;
}

.simple-table tr td:first-child {
    font-weight: bold;
    text-align: right;
}
</style>
