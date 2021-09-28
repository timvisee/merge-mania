<template>
  <div>
    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">Home</h1>
        <p>
            TODO: home page
        </p>

    </div>

  </div>
</template>

<script>
export default {
  name: "Home",
  created() {
    this.onShow();
  },
  watch: {
    $route: "onShow"
  },
  methods: {
    onShow() {
        // Redirect to game if authenticated
        this.$auth
            .isAuth()
            .then((auth) => {
                if(auth)
                    this.$router.push({name: "game"});
            });
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
