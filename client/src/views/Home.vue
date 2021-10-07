<template>
  <div>
    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div class="page-small-card text-center mt-5">

        <h1 class="h3 mb-3 fw-normal">Merge Mania</h1>

        <b-button
            v-if="!auth.auth"
            type="button"
            size="lg"
            variant="primary"
            class="w-100 mt-4"
            @click.prevent.stop="$router.push({name: 'login'})"
        >Login</b-button>

        <b-button
            v-if="auth.auth && auth.hasRoleGame()"
            type="button"
            size="lg"
            variant="primary"
            class="w-100 mt-4"
            @click.prevent.stop="$router.push({name: 'game'})"
        >Game</b-button>

        <b-button
            v-if="auth.auth && auth.hasRoleAdmin()"
            type="button"
            size="lg"
            variant="dark"
            class="w-100 mt-4"
            @click.prevent.stop="$router.push({name: 'admin'})"
        >Admin</b-button>

        <b-button
            type="button"
            variant="link"
            class="w-100 mt-4"
            @click.prevent.stop="$router.push({name: 'about'})"
        >About</b-button>

    </div>

  </div>
</template>

<script>
export default {
  name: "Home",
  data() {
    return {
      auth: this.$auth,
    };
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
</style>
