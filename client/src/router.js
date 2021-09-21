import Vue from "vue";
import Router from "vue-router";
import Home from "./views/Home.vue";
import Login from "./views/Login.vue";
import Game from "./views/Game.vue";

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
    // {
    //   path: "/people",
    //   name: "people",
    //   component: () =>
    //     import(/* webpackChunkName: "people" */ "./views/People.vue")
    // },
  ]
});
