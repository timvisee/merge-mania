<template>
  <div>
    <loader v-if="!app.ready || !app.game || !app.game.ready" />

    <div v-else class="text-center">

        <!-- Error indicator -->
        <b-alert v-if="error" show variant="danger">
            {{ error }}
        </b-alert>

        <h1 class="h3 mb-3 fw-normal">Scanner</h1>

        <qrcode-stream class="viewer" :camera="camera" :track="paintOutline" @decode="onDecode" @init="onInit">

            <div v-if="validating" class="viewer-overlay">
                <loader />
            </div>

            <div v-if="success" class="viewer-overlay">
                <img src="/sprites/clapping-hands.png" alt="Success" />
            </div>

            <div v-if="failed" class="viewer-overlay">
                <img src="/sprites/cross-mark.png" alt="Failed" />
            </div>

        </qrcode-stream>

        <b-button
            type="button"
            size="lg"
            variant="outline-dark"
            class="w-100"
            @click.prevent.stop="showGame"
            squared
            :disabled="success"
        >
            Back to game
        </b-button>

    </div>
  </div>
</template>

<script>
import axios from "axios";
import { QrcodeStream } from 'vue-qrcode-reader'

export default {
  name: "Stats",
  data() {
    return {
      app: this.$app,
      camera: 'auto',
      error: null,

      validating: false,
      success: false,
      failed: false,
    };
  },
  components: {
    QrcodeStream,
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

            // Enable camera on show
            this.setCameraEnabled(true);
        });
  },
  methods: {
    /**
     * Set whether the camera is enabled.
     */
    setCameraEnabled(enabled) {
        this.camera = enabled ? 'auto' : 'off'
    },

    onDecode(result) {
        this.setCameraEnabled(false);
        this.validating = true;

        // Set up listener for QR code scan result
        this.app.socket.addListener('code_result', (valid) => {
            this.validating = false;
            if(valid) {
                this.onSuccess();
            } else {
                this.onFail();
            }
        });

        // Submit scan result
        this.app.socket.send('action_scan_code', result);
    },

    onSuccess() {
        this.success = true;

        setTimeout(() => {
            this.success = false;
            this.showGame();
        }, 1000);
    },

    onFail() {
        this.failed = true;

        setTimeout(() => {
            this.setCameraEnabled(true);
            this.failed = false;
        }, 2000);
    },

    async onInit(promise) {
        try {
            await promise
        } catch (error) {
            if (error.name === 'NotAllowedError') {
                this.error = "ERROR: you need to grant camera access permission"
            } else if (error.name === 'NotFoundError') {
                this.error = "ERROR: no camera on this device"
            } else if (error.name === 'NotSupportedError') {
                this.error = "ERROR: secure context required (HTTPS, localhost)"
            } else if (error.name === 'NotReadableError') {
                this.error = "ERROR: is the camera already in use?"
            } else if (error.name === 'OverconstrainedError') {
                this.error = "ERROR: installed cameras are not suitable"
            } else if (error.name === 'StreamApiNotSupportedError') {
                this.error = "ERROR: Stream API is not supported in this browser"
            } else if (error.name === 'InsecureContextError') {
                this.error = 'ERROR: Camera access is only permitted in secure context. Use HTTPS or localhost rather than HTTP.';
            } else {
                this.error = `ERROR: Camera error (${error.name})`;
            }
        }
    },

    /**
     * Paint QR code outline.
     */
    paintOutline(detectedCodes, ctx) {
      for(const detectedCode of detectedCodes) {
        const [ firstPoint, ...otherPoints ] = detectedCode.cornerPoints

        ctx.lineWidth = 3;
        ctx.strokeStyle = "red";

        ctx.beginPath();
        ctx.moveTo(firstPoint.x, firstPoint.y);
        for (const { x, y } of otherPoints) {
          ctx.lineTo(x, y);
        }
        ctx.lineTo(firstPoint.x, firstPoint.y);
        ctx.closePath();
        ctx.stroke();
      }
    },

    timeout(ms) {
      return new Promise(resolve => {
        window.setTimeout(resolve, ms)
      })
    },

    showGame() {
        this.$router.push({name: "game"});
    },

    redirectToLogin() {
        this.$router.push({name: "login"});
    },
  },
};
</script>

<style scoped>
.viewer {
    border: 1px solid #343a40;
}

.viewer-overlay {
  position: absolute;
  width: 100%;
  height: 100%;

  background-color: rgba(0, 0, 0, .7);
  text-align: center;
  font-weight: bold;
  font-size: 1.4rem;
  padding: 10px;

  display: flex;
  flex-flow: column nowrap;
  justify-content: center;
  align-items: center;
  align-content: center;
}

</style>
