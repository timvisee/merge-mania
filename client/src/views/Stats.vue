<template>
  <div>
    <loader v-if="!app.ready || !app.game || !app.game.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <b-form-select
            v-if="auth.auth && auth.hasRoleAdmin() && users"
            v-model="selectedUser"
            :options="users"
            placeholder="Select a team"
            id="stats-team"
            class="mb-4"
            size="lg"
        ></b-form-select>

        <h1 class="h3 mb-3 fw-normal">Stats</h1>

        <loader v-if="refreshing" />
        <table v-if="stats" class="simple-table">
            <tr><td>Merges:</td><td>{{ stats.merge_count }}</td></tr>
            <tr><td>Buys:</td><td>{{ stats.buy_count }}</td></tr>
            <tr><td>Sells:</td><td>{{ stats.sell_count }}</td></tr>
            <tr><td>Swaps:</td><td>{{ stats.swap_count }}</td></tr>
            <tr><td>Codes:</td><td>{{ stats.code_count }}</td></tr>
            <tr><td>Drops:</td><td>{{ stats.drop_count }}</td></tr>
            <tr><td>Money spent:</td><td>{{ stats.money_spent }}</td></tr>
            <tr><td>Money earned:</td><td>{{ stats.money_earned }}</td></tr>
            <tr><td>Energy spent:</td><td>{{ stats.energy_spent }}</td></tr>
            <tr><td>Energy earned:</td><td>{{ stats.energy_earned }}</td></tr>
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
      auth: this.$auth,
      selectedUser: null,
      loadingUsers: true,
      users: [],
      stats: null,
      refreshing: false,
    };
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

            // Attach stats message listener
            this.$app.socket.addListener('stats', (data) => this.onStats(data));

            // Load list of users if user is admin
            if(this.$auth.hasRoleAdmin())
                this.loadUsers();
            else
                this.loadingUsers = false;

            // Refresh stats if not yet fetched
            if(this.stats == null)
                this.refresh();
        });
  },
  watch: {
    selectedUser: function() {
        this.stats = null;
        this.refresh();
    },
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    refresh() {
        // Fetch stats
        this.refreshing = true;
        this.app.socket.send('get_stats', this.selectedUser);
    },

    onStats(data) {
        this.stats = data;
        this.refreshing = false;
    },

    // Load users to show in form
    loadUsers() {
        this.loadingUsers = true;

        // Request users
        axios.get("/api/auth/users")
            .then(response => {
                // Transform list of users into form select model
                this.users = response.data.map((user) => {
                    return {
                        value: user.id,
                        text: user.name,
                    };
                });
                this.users.unshift({ value: null, text: '----------', disabled: true });
                this.users.unshift({ value: null, text: 'My stats' });
            })
            .catch(err => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loadingUsers = false;
            });
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
