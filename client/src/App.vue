<template>
    <main>

        <!-- Nav bar -->
        <b-navbar toggleable="lg" type="dark" variant="info">
            <b-container>
                <b-navbar-brand href="#">Merge Mania</b-navbar-brand>

                <b-collapse id="nav-collapse" is-nav>
                    <b-navbar-nav>

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

                <!-- TODO: expand button on small screens -->

            </b-container>
        </b-navbar>

        <router-view />

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
