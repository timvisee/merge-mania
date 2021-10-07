<template>
  <div>
    <loader v-if="!app.ready || !app.game || !app.game.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">Stats</h1>

        <table v-if="app.game.stats" class="simple-table">
            <tr><td>Merges:</td><td>{{ app.game.stats.merge_count }}</td></tr>
            <tr><td>Buys:</td><td>{{ app.game.stats.buy_count }}</td></tr>
            <tr><td>Sells:</td><td>{{ app.game.stats.sell_count }}</td></tr>
            <tr><td>Swaps:</td><td>{{ app.game.stats.swap_count }}</td></tr>
            <tr><td>Codes:</td><td>{{ app.game.stats.code_count }}</td></tr>
            <tr><td>Drops:</td><td>{{ app.game.stats.drop_count }}</td></tr>
            <tr><td>Money spent:</td><td>{{ app.game.stats.money_spent }}</td></tr>
            <tr><td>Money earned:</td><td>{{ app.game.stats.money_earned }}</td></tr>
            <tr><td>Energy spent:</td><td>{{ app.game.stats.energy_spent }}</td></tr>
            <tr><td>Energy earned:</td><td>{{ app.game.stats.energy_earned }}</td></tr>
        </table>

        <b-button
            type="button"
            size="lg"
            variant="primary"
            class="w-100 mt-4"
            @click.prevent.stop="refresh"
            :disabled="refreshing"
        >
            <b-spinner v-if="refreshing" small type="grow"></b-spinner>
            <span v-else>Refresh</span>
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
      refreshing: false,
    };
  },
  watch: {
    'app.game.stats': function() {
        this.refreshing = false
    },
  },
  created() {
    // Check auth, initialize game or redirect to login
    this.$auth
        .isAuth()
        .then((auth) => {
            // User must have game role
            if(this.$auth.hasRoleGame()) {
                this.$app.init(this);
                this.$app.initGame();
            } else {
                this.redirectToLogin();
                return;
            }

            // Refresh stats if not yet fetched
            if(this.app.game.stats == null)
                this.refresh();
        });
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    refresh() {
        // Fetch stats
        this.refreshing = true;
        this.app.socket.send('get_stats', null);
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
