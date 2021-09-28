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

        <b-button-group size="lg w-100">
            <b-button type="button" squared variant="outline-dark" class="w-100" @click.stop="toggleMode('merge')" :pressed="mode == 'merge'">Merge</b-button>
            <b-button type="button" squared variant="outline-dark" class="w-100" @click.stop="toggleMode('buy')"  :pressed="mode == 'buy'">Buy</b-button>
            <b-button type="button" squared variant="outline-dark" class="w-100" @click.stop="toggleMode('sell')"  :pressed="mode == 'sell'">Sell</b-button>
            <b-button type="button" squared variant="outline-dark" class="w-100" @click.stop="toggleMode('details')"  :pressed="mode == 'details'">Details</b-button>
        </b-button-group>

        <!-- Inventory grid -->
        <div class="game-grid">
            <div v-for="(cell, index) in game.inventory.grid.items"
                class="cell"
                @click.stop="toggleSelect(index)"
                v-bind:class="{ select: selected == index, item: cell, subtle: merging && !canMerge(index) }"
            >
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
      selected: null,
      mode: null,
      merging: false,
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

    /**
     * Toggle selection of given cell index.
     */
    toggleSelect(index) {
        // In merge mode, merge when selecting second one
        if(this.selected !== null && index !== null && this.selected !== index && this.mode == 'merge' && this.canMerge(index))
            this.actionMerge(this.selected, index);

        // Set selected
        this.selected = this.selected !== index ? index : null;

        // If item is selected, handle other modes
        if(this.selected !== null) {
            switch(this.mode) {
                case 'merge':
                    let isItem = !!this.game.inventory.grid.items[index];
                    this.merging = isItem;
                    break;
                case 'buy':
                    this.actionBuy(index);
                    break;
                case 'sell':
                    this.actionSell(index);
                    break;
                case 'details':
                    this.actionDetails(index);
                    break;
            }
        } else {
            this.merging = false
        }
    },

    actionMerge(index, otherIndex) {
        let isItem = !!this.game.inventory.grid.items[index];
        let isOtherItem = !!this.game.inventory.grid.items[otherIndex];

        // We must merge two items
        // TODO: do not allow to merge items of different types
        if(!isItem || !isOtherItem)
            return;

        alert('TODO: merge items');

        // Reset selectoin
        this.selected = null;
    },

    actionBuy(index) {
        let isItem = !!this.game.inventory.grid.items[index];
        if(!isItem)
            alert('TODO: buy item');
    },

    actionSell(index) {
        let isItem = !!this.game.inventory.grid.items[index];
        if(isItem)
            alert('TODO: sell item');
    },

    actionDetails(index) {
        let isItem = !!this.game.inventory.grid.items[index];
        if(isItem)
            alert('TODO: show item details');
    },

    /**
     * Check whether the given cell index can be merged with the selected cell.
     */
    canMerge(index) {
        // Must be selected
        if(this.selected == null)
            return false;

        // An item must be selected
        let isItem = !!this.game.inventory.grid.items[this.selected];
        if(!isItem)
            return false;

        let a = this.game.inventory.grid.items[this.selected];
        let b = this.game.inventory.grid.items[index];
        return this.itemsTypeEqual(a, b);
    },

    /**
     * Check whether two cells have equal tier and level.
     */
    itemsTypeEqual(a, b) {
        if(a == null || b == null)
            return false;

        if(a.Product && b.Product)
            return a.Product.level == b.Product.level;
        if(a.Factory && b.Factory)
            return a.Factory.level == b.Factory.level;

        return false;
    },

    /**
     * Toggle the current mode.
     */
    toggleMode(mode) {
        this.selected = null;
        this.mode = this.mode !== mode ? mode : null;
        this.merging = false;
    }
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
    margin: 0 auto 2rem auto;
    padding: var(--grid-space);
    grid-template-columns: repeat(var(--grid-row-cells), 1fr);
    grid-template-rows: repeat(var(--grid-row-cells), 1fr);
    gap: var(--grid-space);
    justify-items: stretch;
    align-items: stretch;
    justify-content: stretch;
    align-content: stretch;
    aspect-ratio: 1;
    max-width: 70vh;
    max-height: 70vh;

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
    border-radius: 0.15em;
    text-align: center;
}

.game-grid .cell.select {
    background: #bbb;
}

.game-grid .cell.subtle {
    opacity: 0.5;
}

.game-grid .cell.select.item {
    background: #eb983c;
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
