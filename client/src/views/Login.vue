<template>
  <div>
    <loader v-if="loading" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading" class="page-small-card text-center mt-5">

        <b-form @submit.prevent="onSubmit" @reset.prevent="onReset">

            <h1 class="h3 mb-3 fw-normal">Inloggen</h1>

            <b-form-select
                v-model="form.team"
                id="team"
                :options="teams"
                placeholder="Team"
                class="mt-3"
                size="lg"
            >
                <b-form-select-option :value="null" disabled="disabled">Selecteer team</b-form-select-option>
            </b-form-select>

            <b-form-input
                v-model="form.password"
                id="password"
                type="password"
                placeholder="Wachtwoord"
                class="mt-2"
                size="lg"
            ></b-form-input>

            <b-button
                type="submit"
                size="lg"
                variant="primary"
                class="w-100 mt-4"
            >Inloggen</b-button>

            <b-button
                type="reset"
                variant="link"
                class="w-100 mt-2"
            >Reset</b-button>

        </b-form>

    </div>
  </div>
</template>

<script>
import axios from "axios";

export default {
  name: "Login",
  data() {
    return {
      form: {
        team: null,
        password: null,
      },
      loading: true,
      teams: [],
    };
  },
  created() {
    this.onPageShow();
  },
  watch: {
    $route: "onPageShow"
  },
  methods: {
    // Invoked when page is shown
    onPageShow() {
        this.loading = true;

        // Check whether we're authenticated
        this.$auth
            .checkAuth()
            .then((auth) => {
                if(auth)
                    this.showGame();
                else
                    this.loadTeams();
            })
            .catch((err) => {
                // TODO: remove this line below!
                alert(err);

                // TODO: improve error handling
                alert("Error: " + err.response.data.message);
            });
    },

    // Load teams to show in form
    loadTeams() {
        // Request teams
        axios.get("/api/auth/teams")
            .then(response => {
                // Transform list of teams into form select model
                this.teams = response.data.map((team) => {
                    return {
                        value: team.id,
                        text: team.name,
                    };
                });
            })
            .catch(err => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loading = false;
                this.onReset();
            });
    },

    // Submit form and authenticate
    onSubmit() {
        this.doAuth();
    },

    // Reset form
    onReset() {
        this.form.team = null;
        this.form.password = null;
    },

    // Attempt to authenticate with form data.
    doAuth() {
        this.loading = true;
        this.$auth.login(this.form)
            .then(() => this.showGame())
            .catch(() => {
                // TODO: improve error message
                alert("Error: " + error.response.data.message);
            })
            .finally(() => this.loading = false);
    },

    // Navigate to game page
    showGame() {
        this.$router.push({name: "game"});
    }
  }
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
</style>
