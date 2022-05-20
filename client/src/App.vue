<template>
    <main>

        <!-- Nav bar -->
        <b-navbar toggleable="md" type="dark" variant="dark">
            <b-navbar-brand href="#">Merge Mania</b-navbar-brand>

            <b-navbar-toggle target="navbar-toggle-collapse">
                <template #default="{ expanded }">
                    <b-icon v-if="expanded" icon="chevron-bar-up"></b-icon>
                    <b-icon v-else icon="chevron-bar-down"></b-icon>
                </template>
            </b-navbar-toggle>

            <b-collapse id="navbar-toggle-collapse" is-nav>
                <b-navbar-nav class="ml-auto">
                    <router-link :to="{name: 'index'}" class="nav-link">
                        Home
                    </router-link>
                    <router-link v-if="!auth.auth" :to="{name: 'login'}" class="nav-link">
                        Login
                    </router-link>
                    <router-link v-if="auth.auth && auth.hasRoleGame()" :to="{name: 'game'}" class="nav-link">
                        Game
                    </router-link>
                    <router-link v-if="auth.auth" :to="{name: 'stats'}" class="nav-link">
                        Stats
                    </router-link>
                    <router-link v-if="auth.auth && auth.hasRoleAdmin()" :to="{name: 'leaderboard'}" class="nav-link">
                        Leaderboard
                    </router-link>
                    <router-link v-if="auth.auth && auth.hasRoleAdmin()" :to="{name: 'admin'}" class="nav-link">
                        Admin
                    </router-link>
                    <router-link :to="{name: 'about'}" class="nav-link">
                        About
                    </router-link>
                    <a v-if="auth.auth" href="#" @click.prevent="logout" class="nav-link">
                        Logout
                    </a>
                </b-navbar-nav>
            </b-collapse>

        </b-navbar>

        <b-container class="py-3">
            <router-view />
        </b-container>

    </main>
</template>

<script>
export default {
  name: "app",
  data() {
    return {
      auth: this.$auth,
    };
  },
  created() {
    // Redirect to login page if not authenticated
    this.$auth.isAuth()
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
        this.$auth.logout();
        //this.redirectToLogin();

        // Force refresh to reset client states
        // TODO: this is a hack, improve this, reset states from JS instead
        let loc = window.location;
        window.location.href = loc.origin + loc.pathname;
    },
  },
};
</script>
