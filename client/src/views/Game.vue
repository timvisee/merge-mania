<template>
  <div>
    <loader v-if="!game || !game.ready" />

    <div v-if="error" class="error">
      {{ error }}
    </div>

    <div v-if="game && game.ready" class="text-center">
        <h1 class="h3 mb-3 fw-normal">Game</h1>

        <h5 class="h5 mb-3 fw-normal text-right float-right">
            Energie:
            {{ game.inventory.energy }}
        </h5>
        <h5 class="h5 mb-3 fw-normal text-left">
            Geld:
            {{ game.inventory.money }}
        </h5>

        <!-- Inventory grid -->
        <div class="game-grid">
            <div v-for="cell in game.inventory.grid.items" class="cell">
                <div v-if="cell && cell.Product">
                    <img :src="'/sprites/' + cell.Product.sprite"
                        :title="cell.Product.name"
                        :alt="cell.Product.name"
                        draggable="false"
                    />
                </div>
                <div v-if="cell && cell.Factory">
                    <img :src="'/sprites/' + cell.Factory.sprite"
                        :title="cell.Factory.name"
                        :alt="cell.Factory.name"
                        draggable="false"
                    />
                </div>
            </div>
        </div>

        <b-button-group size="lg w-100">
            <b-button type="button" variant="success" class="w-100">Samenvoegen</b-button>
            <b-button type="button" variant="info" class="w-100">Details</b-button>
        </b-button-group>

        <b-button-group size="lg w-100">
            <b-button type="button" variant="success" class="w-100">Kopen</b-button>
            <b-button type="button" variant="info" class="w-100">Verkopen</b-button>
        </b-button-group>

    </div>
  </div>
</template>

<script>
import sessionManager from "../util/session.js";

export default {
  name: "Game",
  data() {
    return {
      game: this.$game,
    };
  },
  created() {
    this.onRouteChange();

    // Check auth, initialize game or redirect to login
    this.$auth
        .isAuth()
        .then((auth) => {
            if(auth)
                this.$game.init();
            else
                this.redirectToLogin();
        });
  },
  watch: {
    $route: "onRouteChange"
  },
  methods: {
    onRouteChange() {},
    redirectToLogin() {
        this.$router.push({name: "login"});
    },
  }
};
</script>

<style scoped>
.game-grid {
    --grid-space: 5px;
    --grid-row-cells: 8;
}

@media screen and (max-width: 560px) {
    .game-grid {
        --grid-space: 4px;
    }
}

@media screen and (max-width: 470px) {
    .game-grid {
        --grid-space: 3px;
    }
}

@media screen and (max-width: 390px) {
    .game-grid {
        --grid-space: 2px;
    }
}

@media screen and (max-width: 320px) {
    .game-grid {
        --grid-space: 1px;
    }
}

.game-grid {
    display: grid;
    margin: 2rem auto;
    padding: var(--grid-space);
    grid-template-columns: repeat(var(--grid-row-cells), 1fr);
    grid-template-rows: repeat(var(--grid-row-cells), 1fr);
    gap: var(--grid-space);
    justify-items: stretch;
    align-items: stretch;
    justify-content: stretch;
    align-content: stretch;
    aspect-ratio: 1;

    border: black solid 1px;
    box-sizing: content-box;
    background: #eee;

    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -o-user-select: none;
    user-select: none;
}

.game-grid .cell {
    display: inline-block;
    aspect-ratio: 1;
    padding: var(--grid-space);
    box-sizing: content-box;

    border: brown dashed 1px;
    text-align: center;
}

.game-grid .cell img {
    width: 100%;
    height: 100%;
}
</style>

<style>
body {
    background: #f6eada;
}

.game-grid {
    background: #dab382;
}
</style>
