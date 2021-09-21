<template>
  <div id="app">

    <main>

        <!-- Nav bar -->
        <b-navbar toggleable="lg" type="dark" variant="info">
            <b-container>
                <b-navbar-brand href="#">Merge Mania</b-navbar-brand>

                <b-collapse id="nav-collapse" is-nav>
                    <b-navbar-nav>

                        <router-link to="/login" class="nav-link">
                            Login
                        </router-link>
                        <router-link to="/game" class="nav-link">
                            Game
                        </router-link>

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

  </div>
</template>

<script>
export default {
  name: "app"
};

// TODO: redirect to login route if not authenticated
</script>

<style scoped>
#app {
  margin: 0;
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  padding: 0;
}

footer {
  padding: 5rem 1rem;
  text-align: center;
  font-size: 1rem;
}
</style>
