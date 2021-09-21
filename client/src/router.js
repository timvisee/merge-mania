import Vue from "vue";
import Router from "vue-router";
import Home from "./views/Home.vue";
import Game from "./views/Game.vue";

Vue.use(Router);

export default new Router({
  mode: "history",
  base: process.env.BASE_URL,
  routes: [
    {
      path: "/",
      name: "index",
      component: Home,
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
