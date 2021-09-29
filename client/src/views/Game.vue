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

        <b-button-group class="tabs w-100" size="lg">
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                squared
                @click.stop.prevent="toggleMode('merge')"
                :pressed="mode == 'merge'">Merge</b-button>
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                squared
                @click.prevent="toggleMode('buy')"
                :pressed="mode == 'buy'">Buy</b-button>
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                squared
                @click.prevent="toggleMode('sell')"
                :pressed="mode == 'sell'">Sell</b-button>
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                squared
                @click.prevent="toggleMode('details')"
                :pressed="mode == 'details'">Details</b-button>
        </b-button-group>

        <!-- Inventory grid -->
        <div class="game-grid">
            <div v-for="(cell, index) in game.inventory.grid.items"
                class="cell"
                @click.stop="toggleSelect(index)"
                v-bind:class="{ select: selected == index, item: cell, factory: cell && cell.Factory, subtle: isSubtle(index) }"
            >
                <div v-if="cell && cell.Product">
                    <div v-if="mode == 'sell'" class="overlay">
                        <div v-if="cell.Product.sell_price" class="nw">
                            {{ cell.Product.sell_price }}
                        </div>
                    </div>
                    <img :src="'/sprites/' + cell.Product.sprite"
                        :title="cell.Product.name"
                        :alt="cell.Product.name"
                        draggable="false"
                    />
                </div>
                <div v-if="cell && cell.Factory">
                    <div class="overlay">
                        <div v-if="mode == 'details'" class="ne">
                            {{ cell.Factory.interval }}t
                        </div>
                        <div v-if="mode == 'sell' && cell.Factory.sell_price" class="nw">
                            {{ cell.Factory.sell_price }}
                        </div>
                    </div>
                    <img :src="'/sprites/' + cell.Factory.sprite"
                        :title="cell.Factory.name"
                        :alt="cell.Factory.name"
                        draggable="false"
                    />
                    <div class="overlay">
                        <div class="sw">
                            {{ cell.Factory.level + 1 }}
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Buy modal -->
        <b-modal
            id="game-buy-modal"
            title="Buy factory"
            @hidden="selected = null"
            centered
            no-fade
        >
            <div>
                <b-button @click.stop.prevent="doBuy(selected, '101.0')">Tree</b-button>
            </div>

            <div>
                <b-button @click.stop.prevent="doBuy(selected, '102.0')">Chicken</b-button>
            </div>

            <template #modal-footer="{ cancel }">
                <b-button variant="secondary" @click="cancel()">
                    Close
                </b-button>
            </template>
        </b-modal>

        <!-- Details modal -->
        <b-modal
            id="game-details-modal"
            title="Item details"
            @hidden="selected = null"
            centered
            no-fade
        >
            <div v-if="selectedCell && selectedCell.Product" class="text-center">
                <img :src="'/sprites/' + selectedCell.Product.sprite"
                    :title="selectedCell.Product.name"
                    :alt="selectedCell.Product.name"
                    draggable="false"
                />
                <table>
                    <tr><td>Type:</td><td>Product</td></tr>
                    <tr><td>Name:</td><td>{{ selectedCell.Product.name }}</td></tr>
                    <tr><td>Tier:</td><td>{{ selectedCell.Product.tier }}</td></tr>
                    <tr><td>Level:</td><td>{{ selectedCell.Product.level + 1 }}</td></tr>
                    <tr><td>Sell price:</td><td>{{ selectedCell.Product.sell_price }}</td></tr>
                </table>
            </div>

            <div v-if="selectedCell && selectedCell.Factory" class="text-center">
                <img :src="'/sprites/' + selectedCell.Factory.sprite"
                    :title="selectedCell.Factory.name"
                    :alt="selectedCell.Factory.name"
                    draggable="false"
                />
                <table>
                    <tr><td>Type:</td><td>Factory</td></tr>
                    <tr><td>Name:</td><td>{{ selectedCell.Factory.name }}</td></tr>
                    <tr><td>Tier:</td><td>{{ selectedCell.Factory.tier }}</td></tr>
                    <tr><td>Level:</td><td>{{ selectedCell.Factory.level + 1 }}</td></tr>
                    <tr><td>Production interval:</td><td>1 / {{ selectedCell.Factory.interval }} ticks</td></tr>
                    <tr><td>Sell price:</td><td>{{ selectedCell.Factory.sell_price }}</td></tr>
                </table>
            </div>

            <template #modal-footer="{ cancel }">
                <b-button variant="secondary" @click="cancel()">
                    Close
                </b-button>
            </template>
        </b-modal>

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
      mode: null,
      selected: null,
      selectedCell: null,
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
     * Toggle the current mode.
     */
    toggleMode(mode) {
        this.selected = null;
        this.mode = this.mode !== mode ? mode : null;
    },

    /**
     * Toggle selection of given cell index.
     */
    toggleSelect(index) {
        // In merge mode, merge when selecting second one
        if(this.selected !== null && index !== null && this.selected !== index && this.mode == 'merge' && this.canMerge(index)) {
            this.actionMerge(this.selected, index);
            return;
        }

        // Update selected
        this.selected = this.selected !== index ? index : null;

        // If a cell is selected, invoke an action
        if(this.selected !== null) {
            switch(this.mode) {
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
        }
    },

    actionMerge(index, otherIndex) {
        // We must merge two items
        // TODO: do not allow to merge items of different types
        if(!this.hasItem(index) || !this.hasItem(otherIndex))
            return;

        // Send merge action
        this.$game.socket.send('action_merge', {
            cell: otherIndex,
            other: index,
        });

        // Reset selection, clear cell for instant feedback
        this.$game.inventory.grid.items[index] = null;
        this.selected = null;
    },

    actionBuy(index) {
        // Cell must be empty
        if(this.hasItem(index))
            return;

        // Show buy modal
        this.$bvModal.show('game-buy-modal');
    },

    doBuy(index, item) {
        this.$bvModal.hide('game-buy-modal');

        // Send buy action
        this.$game.socket.send('action_buy', {
            cell: index,
            item,
        });
    },

    actionSell(index) {
        // Cell must not be empty
        if(!this.hasItem(index))
            return;

        // Send sell action
        this.$game.socket.send('action_sell', {
            cell: index,
        });

        // Reset selection, clear cell for instant feedback
        this.$game.inventory.grid.items[index] = null;
        this.selected = null;
    },

    actionDetails(index) {
        // Cell must not be empty
        if(!this.hasItem(index))
            return;

        // Show details modal
        this.selectedCell = this.$game.inventory.grid.items[index];
        this.$bvModal.show('game-details-modal');
    },

    /**
     * Whether a cell should be shown as subtle.
     */
    isSubtle(index) {
        switch(this.mode) {
            case 'merge':
                // A cell with no item is always subtle
                if(!this.hasItem(index))
                    return true;

                // Must have mergeable state
                let a = this.game.inventory.grid.items[index];
                if((a.Product && !a.Product.can_upgrade) || (a.Factory && !a.Factory.can_upgrade))
                    return true;

                // A cell with an item must be selected
                if(!this.hasItem(this.selected))
                    return false;

                // Show as subtle if can't merge
                return !this.canMerge(index);
            case 'buy':
                return this.hasItem(index);
            case 'sell':
            case 'details':
                return !this.hasItem(index);
            defaut:
                return false;
        }
    },

    /**
     * Check whether a cell by index has an item.
     */
    hasItem(index) {
        return index !== null && !!this.game.inventory.grid.items[index];
    },

    /**
     * Check whether the given cell index can be merged with the other cell.
     * Uses selected cell by default.
     */
    canMerge(index, other) {
        other = other !== undefined ? other : this.selected;

        // Cells must be items
        if(!this.hasItem(index) || !this.hasItem(other))
            return false;

        let a = this.game.inventory.grid.items[other];
        let b = this.game.inventory.grid.items[index];

        // Must have mergeable state
        if((a.Product && !a.Product.can_upgrade) || (a.Factory && !a.Factory.can_upgrade))
            return false;

        return this.isEqualItemType(a, b);
    },

    /**
     * Check whether two cells have equal tier and level.
     */
    isEqualItemType(a, b) {
        if(a == null || b == null)
            return false;

        // Must both be products or factories
        if(a.Product && b.Product) {
            a = a.Product;
            b = b.Product;
        } else if(a.Factory && b.Factory) {
            a = a.Factory;
            b = b.Factory;
        } else
            return false;

        // Tier and level must equal
        return a.tier == b.tier && a.level == b.level;
    },
  }
};
</script>

<style scoped>
.tabs {
    overflow-x: auto;
}

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

.game-grid .cell.factory {
    background: lightblue;
}

.game-grid .cell.select {
    background: #bbb;
}

.game-grid .cell.select.item {
    background: #eb983c;
}

.game-grid .cell.subtle {
    opacity: 0.5;
}

.game-grid .cell img {
    width: 100%;
    height: 100%;
}

.game-grid .cell .overlay {
    position: relative;
}

.game-grid .cell .overlay div {
    font-weight: 900;
    font-size: 0.9em;
    -webkit-text-stroke-width: 1px;
    -webkit-text-stroke-color: white;
}

.game-grid .cell .overlay .ne {
    position: absolute;
    right: 0;
    top: -5px;
}

.game-grid .cell .overlay .nw {
    position: absolute;
    left: 0;
    top: -5px;
}

.game-grid .cell .overlay .sw {
    position: absolute;
    left: 0;
    bottom: -5px;
}

// Patch to fix inactive mode button staying highlighted on mobile
.btn-outline-dark:not(.disabled):not(.active):active,
.btn-outline-dark:not(.disabled):not(.active):hover {
    background: transparent;
    color: #343a40;
    border-color: #343a40;
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
