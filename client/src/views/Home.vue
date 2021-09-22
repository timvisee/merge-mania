<template>
  <div>
    <div class="loading" v-if="loading">
      Loading...
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>
  </div>
</template>

<script>
import auth from "../auth";

export default {
  name: "Home",
  data() {
    return {
      loading: true,
    };
  },
  created() {
    this.onShow();
  },
  watch: {
    $route: "onShow"
  },
  methods: {
    onShow() {
        // Redirect to game if authenticated
        auth.isAuth()
            .then((auth) => {
                if(auth)
                    this.$router.push({name: "game"});
            });
    }
  }
};
</script>
