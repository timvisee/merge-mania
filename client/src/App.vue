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
                    <router-link :to="{name: 'index'}" class="nav-link">
                        Home
                    </router-link>
                    <router-link :to="{name: 'login'}" class="nav-link">
                        Inloggen
                    </router-link>
                    <router-link :to="{name: 'game'}" class="nav-link">
                        Game
                    </router-link>
                    <router-link :to="{name: 'about'}" class="nav-link">
                        Over
                    </router-link>
                    <a href="#" @click.prevent="logout" class="nav-link">
                        Uitloggen
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
        // TODO: invalidate current session on server
        auth.resetSessionToken();
        this.redirectToLogin();
    },
  },
};
</script>
