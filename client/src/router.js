import Vue from "vue";
import Router from "vue-router";
import Home from "./views/Home.vue";
import Login from "./views/Login.vue";
import Game from "./views/Game.vue";
import Scan from "./views/Scan.vue";
import Stats from "./views/Stats.vue";
import Leaderboard from "./views/Leaderboard.vue";
import Admin from "./views/Admin.vue";
import About from "./views/About.vue";

Vue.use(Router);

export default new Router({
  mode: "hash",
  // TODO: switch to history mode once configured in Rocket
  // mode: "history",
  base: process.env.BASE_URL,
  routes: [
    {
      path: "/",
      name: "index",
      component: Home,
    },
    {
      path: "/login",
      name: "login",
      component: Login,
    },
    {
      path: "/game",
      name: "game",
      component: Game,
    },
    {
      path: "/scan",
      name: "scan",
      component: Scan,
    },
    {
      path: "/stats",
      name: "stats",
      component: Stats,
    },
    {
      path: "/leaderboard",
      name: "leaderboard",
      component: Leaderboard,
    },
    {
      path: "/admin",
      name: "admin",
      component: Admin,
    },
    {
      path: "/about",
      name: "about",
      component: About,
    },
    // {
    //   path: "/people",
    //   name: "people",
    //   component: () =>
    //     import(/* webpackChunkName: "people" */ "./views/People.vue")
    // },
  ]
});
