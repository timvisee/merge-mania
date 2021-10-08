<template>
  <div>
    <loader v-if="!app.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">Leaderboard</h1>

        <table v-if="leaderboard" class="simple-table">
            <tr v-for="entry in leaderboard">
                <td>{{ entry.name }}</td>
                <td>{{ entry.money }} <span class="subtle">money</span></td>
            </tr>
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
export default {
  name: "Leaderboard",
  data() {
    return {
      app: this.$app,
      leaderboard: null,
      refreshing: false,
    };
  },
  created() {
    // Check auth, initialize game or redirect to login
    this.$auth
        .isAuth()
        .then((auth) => {
            // User must have admin role
            if(this.$auth.hasRoleAdmin()) {
                this.$app.init(this);
            } else {
                this.redirectToLogin();
                return;
            }

            // Attach leaderboard message listener
            this.$app.socket.addListener('leaderboard', (data) => this.onLeaderboard(data));

            // Refresh leaderboad if not yet fetched
            if(this.leaderboard == null)
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
        this.app.socket.send('get_leaderboard', null);
    },

    onLeaderboard(data) {
        this.leaderboard = data;
        this.refreshing = false;
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

span.subtle {
    color: gray;
}
</style>
