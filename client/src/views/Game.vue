<template>
  <div>
    <div class="loading" v-if="loading">
      Loading...
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div>
        TODO: game content here...
    </div>
  </div>
</template>

<script>
export default {
  name: "Game",
  data() {
    return {
      loading: true,
    };
  },
  created() {
    this.fetchData();
  },
  watch: {
    $route: "fetchData"
  },
  methods: {
    fetchData() {
      this.loading = false;
    }
  }
};
</script>
