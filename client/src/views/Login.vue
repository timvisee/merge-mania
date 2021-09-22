<template>
  <div>
    <div class="loading" v-if="loading">
      Loading...
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading">

        <b-form @submit.prevent="onSubmit" @reset.prevent="onReset">

            <b-form-group
                id="team"
                label="Team:"
                label-for="team"
                required
            >
                <b-form-select
                    v-model="form.team"
                    id="team"
                    :options="teams"
                ></b-form-select>
            </b-form-group>

            <b-form-group
                id="password"
                label="Password:"
                label-for="password"
                description="What's the password?"
                required
            >
                <b-form-input v-model="form.password" id="password" type="password"></b-form-input>
            </b-form-group>

            <b-button type="submit" variant="primary">Inloggen</b-button>
            &nbsp;
            <b-button type="reset" variant="danger">Reset</b-button>

        </b-form>

    </div>
  </div>
</template>

<script>
import axios from "axios";
import auth from "../auth";

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
        auth.isAuth()
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
        this.attemptAuth();
    },

    // Reset form
    onReset() {
        this.form.team = null;
        this.form.password = null;
    },

    // Attempt to authenticate with form data.
    attemptAuth() {
        this.loading = true;
        axios.post("/api/auth/login", this.form)
            .then((response) => {
                auth.setSessionToken(response.data.token);
                this.showGame();
            })
            .catch((error) => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loading = false;
            });
    },

    // Navigate to game page
    showGame() {
        this.$router.push({name: "game"});
    }
  }
};
</script>
