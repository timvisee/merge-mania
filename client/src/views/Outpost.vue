<template>
  <div>
    <loader v-if="!app.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <h1 class="h3 mb-5 fw-normal">{{ outpost.name }}</h1>

        <loader v-if="!token" />

        <qrcode-vue
            v-else
            class="code"
            level="H"
            size="300"
            render-as="svg"
            :value="token"
        ></qrcode-vue>

        <b-button
            type="button"
            size="lg"
            variant="secondary"
            class="w-100 mt-4"
            @click.prevent.stop="showManualDialog"
        >
            Manual Reward
        </b-button>

        <!-- Manual modal -->
        <b-modal
            id="outpost-manual-modal"
            title="Manual Reward"
            centered
        >
            <label for="outpost-team">Team:</label>
            <b-form-select
                v-model="selectedUser"
                :options="users"
                placeholder="Select a team"
                id="outpost-team"
                class="mb-4"
                size="lg"
            ></b-form-select>

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
                @click.prevent.stop="rewardUser"
                :disabled="selectedUser == null || loadingUsers || rewarding"
            >
                <b-spinner v-if="loadingUsers || rewarding" small type="grow"></b-spinner>
                <span v-else>Give Rewards</span>
            </b-button>

            <template #modal-footer="{ cancel }">
                <b-button variant="secondary" @click="cancel()" :disabled="rewarding">
                    Close
                </b-button>
            </template>
        </b-modal>

    </div>
  </div>
</template>

<script>
import axios from "axios";
import QrcodeVue from 'qrcode.vue'

/**
 * QR code refresh interval from server in milliseconds.
 */
const CODE_REFRESH_INTERVAL = 30000;

export default {
  name: "Outpost",
  data() {
    return {
      app: this.$app,
      outpost: {
        name: localStorage.getItem('outpost.name') || 'Outpost',
        id: parseInt(localStorage.getItem('outpost.id')) || 1,
      },
      token: null,
      updateTimer: null,
      selectedUser: null,
      loadingUsers: true,
      users: [],
      rewarding: false,
    };
  },
  components: {
    QrcodeVue,
  },
  watch: {
    token: function() {
        setTimeout(() => this.fixQrSize(), 0);
    },
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

            // Attach stats message listener
            this.$app.socket.addListener('outpost_token', (data) => this.onToken(data));

            // Set up timer to update QR code, update once
            this.setUpTimer();
            this.update();
        });
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    /**
     * Request new QR-code token.
     */
    update() {
        this.app.socket.send('get_outpost_token', this.outpost.id);
    },

    /**
     * Set up timer to periodically update QR code.
     */
    setUpTimer() {
        if(this.updateTimer != null)
            clearInterval(this.updateTimer);

        this.updateTimer = setInterval(() => this.update(), CODE_REFRESH_INTERVAL);
    },

    onToken(token) {
        this.token = token;
    },

    /**
     * Strip enforced QR code sizes to nicely scale it
     */
    fixQrSize() {
        let code = document.querySelector('.code');
        if(code === undefined)
            return;

        let qr = code.childNodes[0];
        if(qr === undefined)
            return;

        qr.removeAttribute("width");
        qr.removeAttribute("height");
        qr.removeAttribute("style");
    },

    showManualDialog() {
        // If user list is emtpy, fetch from server
        if(this.users.length == 0)
            this.loadUsers();

        // Show manual outpost modal
        this.$bvModal.show('outpost-manual-modal');

        this.selectedUser = null;
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
                this.users.unshift({ value: null, text: 'Select a team...', disabled: true });
            })
            .catch(err => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loadingUsers = false;
            });
    },

    // Manually reward the given user
    rewardUser() {
        if(this.selectedUser == null)
            return;

        this.rewarding = true;

        // // Set up listener for reward result
        // this.app.socket.addListener('reward_result', (valid) => {
        //     //
        // });

        // Submit manual reward result
        this.app.socket.send('action_reward_user', {
            outpost_id: this.outpost.id,
            user_id: this.selectedUser,
        });

        this.rewarding = false;

        // Hide manual outpost modal
        this.$bvModal.hide('outpost-manual-modal');
    },
  },
};
</script>

<style scoped>
.code {
    max-width: 70vh;
    margin: 0 auto;
    padding: 3em;
    background: white;
    aspect-ratio: 1.0;

    display: flex;
    justify-content: stretch;
    align-items: stretch;
    align-content: stretch;
}

.code canvas,
.code svg {
    flex-grow: 1;
    align-self: stretch;
    aspect-ratio: 1.0;
}
</style>
