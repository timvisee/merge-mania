<template>
  <div>
    <div class="loading" v-if="loading">
      Loading...
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading">
        <form>

            <label for="team">Team:</label>
            <select name="team" id="team">
                <option v-for="team in teams" value="{{ team.id }}">{{ team.name }}</option>
            </select>

            <label for="password">Password:</label>
            <input type="password" name="password" />

            <input type="submit" name="submit" value="Inloggen" />

        </form>
    </div>
  </div>
</template>

<script>
import axios from "axios";

export default {
  name: "Login",
  data() {
    return {
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
        axios.get("/api/teams")
            .then(response => {
                this.teams = response.data;
            })
            .catch(err => {
                // TODO: report error
            })
            .finally(() => {
                this.loading = false;
            });
    }
  }
};
</script>
