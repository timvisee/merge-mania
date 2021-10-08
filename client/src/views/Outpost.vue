<template>
  <div>
    <loader v-if="!app.ready" />

    <div v-else class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">{{ outpost.name }}</h1>

        <loader v-if="!token" />

        <qrcode-vue
            v-else
            class="code"
            level="H"
            size="300"
            :value="token"
        ></qrcode-vue>

    </div>
  </div>
</template>

<script>
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
    };
  },
  components: {
    QrcodeVue,
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
}

.code canvas {
    width: auto !important;
    height: auto !important;
    aspect-ratio: 1.0;
}
</style>
