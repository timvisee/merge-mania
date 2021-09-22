<template>
  <div>
    <div class="loading" v-if="loading">
      Loading...
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading">
        <b-container fluid="sm" class="py-3">
            <b-form @submit.prevent="onSubmit" @reset="onReset">

                <b-form-group
                    id="team"
                    label="Team:"
                    label-for="team"
                    required
                >
                    <b-form-select
                        v-model="form.team"
                        id="team"
                        class="form-select"
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
                <b-button type="reset" variant="danger">Reset</b-button>

            </b-form>
        </b-container>
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
    this.checkLogin();
  },
  watch: {
    $route: "checkLogin"
  },
  methods: {
    checkLogin() {
        // TODO: redirect to game if authenticated

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
                // TODO: report error
            })
            .finally(() => {
                this.loading = false;
                this.onReset();
            });
    },
    onSubmit() {
        this.attemptLogin();
        return false;
    },
    onReset() {
        this.form.team = null;
        this.form.password = null;
        return false;
    },
    attemptLogin() {
        this.loading = true;
        axios.post("/api/auth/login", this.form)
            .then((response) => {
                // TODO: handle response, set token
                alert(JSON.stringify(response.data));
            })
            .catch((error) => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loading = false;
            });
    },
  }
};
</script>
