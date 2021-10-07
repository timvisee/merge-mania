<template>
  <div>
    <loader v-if="loading" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="!loading" class="page-small-card text-center mt-5">

        <b-form @submit.prevent="onSubmit" @reset.prevent="onReset">

            <h1 class="h3 mb-3 fw-normal">Login</h1>

            <b-form-select
                v-model="form.user"
                id="user"
                :options="users"
                placeholder="User"
                class="mt-3"
                size="lg"
            >
                <b-form-select-option :value="null" disabled="disabled">Select user</b-form-select-option>
            </b-form-select>

            <b-form-input
                v-model="form.password"
                id="password"
                type="password"
                placeholder="Password"
                class="mt-2"
                size="lg"
            ></b-form-input>

            <b-button
                type="submit"
                size="lg"
                variant="primary"
                class="w-100 mt-4"
            >Login</b-button>

            <b-button
                type="reset"
                variant="link"
                class="w-100 mt-2"
            >Reset</b-button>

        </b-form>

    </div>
  </div>
</template>

<script>
import axios from "axios";

export default {
  name: "Login",
  data() {
    return {
      form: {
        user: null,
        password: null,
      },
      loading: true,
      users: [],
    };
  },
  created() {
    this.onPageShow();
  },
  watch: {
    $route: "onPageShow"
  },
  methods: {
    // Invoked when page is shown
    onPageShow() {
        this.loading = true;

        // Check whether we're authenticated
        this.$auth
            .checkAuth()
            .then((auth) => {
                if(this.$auth.auth)
                    this.afterLogin();
                else
                    this.loadUsers();
            })
            .catch((err) => {
                // TODO: remove this line below!
                alert(err);

                // TODO: improve error handling
                alert("Error: " + err.response.data.message);
            });
    },

    // Load users to show in form
    loadUsers() {
        // Request users
        axios.get("/api/auth/users")
            .then(response => {
                // Transform list of users into form select model
                this.users = response.data.map((user) => {
                    return {
                        value: user.id,
                        text: user.name,
                    };
                });
            })
            .catch(err => {
                // TODO: improve error handling
                alert("Error: " + error.response.data.message);
            })
            .finally(() => {
                this.loading = false;
                this.onReset();
            });
    },

    // Submit form and authenticate
    onSubmit() {
        this.doAuth();
    },

    // Reset form
    onReset() {
        this.form.user = null;
        this.form.password = null;
    },

    // Attempt to authenticate with form data.
    doAuth() {
        this.loading = true;
        this.$auth.login(this.form)
            .then(() => this.afterLogin())
            .catch((msg) => {
                // TODO: improve error message
                alert("Error: " + msg);
            })
            .finally(() => this.loading = false);
    },

    // Route user to correct page after login.
    afterLogin() {
        if(this.$auth.hasRoleGame())
            this.showPage("game");
        else if(this.$auth.hasRoleAdmin())
            this.showPage("admin");
        else if(this.$auth.auth)
            this.showPage("index");
    },

    // Navigate to the given page.
    showPage(page) {
        this.$router.push({name: page});
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
