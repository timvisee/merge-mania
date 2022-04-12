<template>
  <div>
    <loader v-if="!app.ready" />

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
            variant="primary"
            class="w-100 mt-4"
            @click.prevent.stop="showOutpostDialog"
        >
            Create outpost
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

        <!-- Outpost modal -->
        <b-modal
            id="admin-outpost-modal"
            title="Create outpost"
            centered
        >
            <label for="outpost-name">Name:</label>
            <b-form-input
                v-model="outpost.name"
                id="name"
                type="text"
                placeholder="My outpost"
                class="mb-4"
                size="lg"
            ></b-form-input>

            <label for="outpost-id">Unique Post ID:</label>
            <b-form-spinbutton
                id="outpost-id"
                v-model="outpost.id"
                min="1"
                max="99999"
                class="w-100 mb-4"
                size="lg"
            ></b-form-spinbutton>

            <b-button
                type="button"
                size="lg"
                variant="primary"
                class="w-100"
                @click.prevent.stop="createOutpost"
            >
                Create outpost
            </b-button>

            <template #modal-footer="{ cancel }">
                <b-button variant="secondary" @click="cancel()">
                    Close
                </b-button>
            </template>
        </b-modal>

    </div>
  </div>
</template>

<script>
export default {
  name: "Stats",
  data() {
    return {
      app: this.$app,
      outpost: {
        name: localStorage.getItem('outpost.name') || null,
        id: parseInt(localStorage.getItem('outpost.id')) || 1,
      },
    };
  },
  created() {
    // Check auth, must be admin or redirect to login
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

    showOutpostDialog() {
        // Show outpost modal
        this.$bvModal.show('admin-outpost-modal');
    },

    createOutpost() {
        // Store outpost configuration
        localStorage.setItem('outpost.id', this.outpost.id);
        localStorage.setItem('outpost.name', this.outpost.name);

        // Show outpost page
        this.$router.push({name: "outpost"});
    },

    reset() {
        // Show confirmation dialog
        this.$bvModal.msgBoxConfirm('This will reset the game and all user inventories. Are you sure you want to continue?', {
            title: 'Reset game?',
            okVariant: 'danger',
            okTitle: 'Reset',
            cancelTitle: 'Cancel',
            footerClass: 'p-2',
            hideHeaderClose: false,
            centered: true
        })
        .then(confirmed => {
            if(!confirmed)
                return;

            // Send reset command
            console.debug("[admin] Reset game");
            this.app.socket.send('reset_game');
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
