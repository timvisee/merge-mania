<template>
    <main>

        <!-- Nav bar -->
        <b-navbar toggleable="sm" type="dark" variant="dark">
            <b-navbar-brand href="#">Merge Mania</b-navbar-brand>

            <b-navbar-toggle target="navbar-toggle-collapse">
                <template #default="{ expanded }">
                    <b-icon v-if="expanded" icon="chevron-bar-up"></b-icon>
                    <b-icon v-else icon="chevron-bar-down"></b-icon>
                </template>
            </b-navbar-toggle>

            <b-collapse id="navbar-toggle-collapse" is-nav>
                <b-navbar-nav class="ml-auto">
                    <router-link :to="{name: 'home'}" class="nav-link">
                        Home
                    </router-link>
                    <router-link :to="{name: 'login'}" class="nav-link">
                        Login
                    </router-link>
                    <router-link :to="{name: 'game'}" class="nav-link">
                        Game
                    </router-link>
                    <a href="#" @click.prevent="logout" class="nav-link">
                        Logout
                    </a>
                </b-navbar-nav>
            </b-collapse>

        </b-navbar>

        <b-container class="py-3">
            <router-view />
        </b-container>

        <footer>
            Made by <a href="https://timvisee.com/" target="_blank">Tim Visée</a>
        </footer>

    </main>
</template>

<script>
import auth from "./auth";

export default {
  name: "app",
  created() {
    // Redirect to login page if not authenticated
    auth.isAuth()
        .then((auth) => {
            if(!auth)
                this.redirectToLogin();
        });
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    logout() {
        // TODO: invalidate current session on server
        auth.resetSessionToken();
        this.redirectToLogin();
    },
  },
};
</script>

<style scoped>
footer {
    padding: 5rem 1rem;
    text-align: center;
    font-size: 1rem;
}
</style>
