<template>
  <div>
    <loader v-if="loading" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div>
        TODO: game content here...
    </div>
  </div>
</template>

<script>
import auth from "../auth";

export default {
  name: "Game",
  data() {
    return {
      loading: true,
    };
  },
  created() {
    // Redirect to login page if not authenticated
    auth.isAuth()
        .then((auth) => {
            if(!auth)
                this.redirectToLogin();
        });

    this.fetchData();
  },
  watch: {
    $route: "fetchData"
  },
  methods: {
    fetchData() {
      this.loading = false;
    },

    redirectToLogin() {
        this.$router.push({name: "login"});
    },
  }
};
</script>
