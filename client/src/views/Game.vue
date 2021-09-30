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
            <div v-for="(cell, index) in game.inventory.items"
                class="cell"
                @click.stop="toggleSelect(index)"
                v-bind:class="{ select: selected == index, item: cell, factory: cell && cell.drop_interval, subtle: isSubtle(index), plus: !cell && mode == 'buy' }"
            >
                <div v-if="cell">
                    <div class="overlay">
                        <div v-if="mode == 'details' && cell.drop_interval" class="ne">
                            {{ cell.drop_interval }}t
                        </div>
                        <div v-if="mode == 'sell' && cell.sell" class="nw">
                            {{ cell.sell }}
                        </div>
                    </div>
                    <img :src="'/sprites/' + cell.sprite"
                        :title="cell.name"
                        :alt="cell.name"
                        draggable="false"
                    />
                    <div v-if="cell.label" class="overlay">
                        <div class="sw">
                            {{ cell.label }}
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <b-button-group class="w-100" size="lg">
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                @click.stop.prevent="actionScanCode"
                squared>Scan code</b-button>
            <b-button
                type="button"
                class="w-100"
                variant="outline-dark"
                @click.stop.prevent="actionCatalog"
                squared>Catalog</b-button>
        </b-button-group>

        <!-- Buy modal -->
        <b-modal
            id="game-buy-modal"
            title="Buy item"
            @hidden="selected = null"
            centered
            no-fade
        >
            <div class="item-list">
                <div v-for="item in game.getBuyableItems()" class="item">
                    <img :src="'/sprites/' + item.sprite"
                        :title="item.name"
                        :alt="item.name"
                        @click.stop.prevent="doBuy(selected, item.ref)"
                        draggable="false"
                    />
                    <div class="overlay">
                        <!-- TODO: better price rendering -->
                        <div v-if="item.buy[0].item" class="sw">
                            {{ item.buy[0].item }}x{{ item.buy[0].quantity }}
                        </div>
                        <div v-else class="sw">
                            {{ item.buy[0] }}
                        </div>
                    </div>
                </div>
            </div>

            <template #modal-footer="{ cancel }">
                <b-button variant="secondary" @click="cancel()">
                    Close
                </b-button>
            </template>
        </b-modal>

        <!-- Details modal -->
        <!-- TODO: instantiate new modal on show, use item config instead of reference -->
        <b-modal
            id="game-details-modal"
            title="Item details"
            @hidden="selected = null"
            centered
            no-fade
        >
            <div v-if="selectedCell" class="text-center">
                <div class="item-list">
                    <div
                        v-for="(item, index) in game.getDownUpgradeItems(selectedCell.ref)"
                        class="item"
                        v-bind:class="{ highlight: item.ref == selectedCell.ref }"
                    >
                        <img v-if="game.isDiscovered(item.ref, selectedCell.ref)"
                            :src="'/sprites/' + item.sprite"
                            :title="item.name"
                            :alt="item.name"
                            draggable="false"
                        />
                        <img v-else
                            src="/sprites/white-question-mark.png"
                            title="Undiscovered item"
                            alt="Undiscovered item"
                            draggable="false"
                        />
                        <div class="overlay">
                            <div class="sw">
                                #{{ index + 1 }}
                            </div>
                        </div>
                    </div>
                </div>

                <table class="simple-table">
                    <tr><td>Name:</td><td>{{ selectedCell.name }}</td></tr>
                    <tr><td>Tier:</td><td>{{ selectedCell.tier }}</td></tr>
                    <tr v-if="selectedCell.label"><td>Label:</td><td>{{ selectedCell.label }}</td></tr>
                    <tr v-if="selectedCell.drop_interval"><td>Production interval:</td><td><span class="subtle">1 / </span>{{ selectedCell.drop_interval }} ticks</td></tr>
                    <!-- TODO: render drops -->
                    <tr v-if="selectedCell.drop_interval"><td>Drops:</td><td><span class="subtle">?</span></td></tr>
                    <tr><td>Sell price:</td><td>{{ selectedCell.sell }}</td></tr>
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
        this.$game.inventory.items[index] = null;
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
        this.$game.inventory.items[index] = null;
        this.selected = null;
    },

    actionDetails(index) {
        // Cell must not be empty
        if(!this.hasItem(index))
            return;

        // Show details modal
        this.selectedCell = this.$game.inventory.items[index];
        this.$bvModal.show('game-details-modal');
    },

    actionScanCode() {
        alert('not yet implemented');
    },

    actionCatalog() {
        alert('not yet implemented');
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
                if(!this.game.inventory.items[index].mergeable)
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
        return index !== null && !!this.game.inventory.items[index];
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

        let a = this.game.inventory.items[other];
        let b = this.game.inventory.items[index];

        // Must be mergeable, must have same ID
        return a.mergeable && a.ref == b.ref;
    },
  }
};
</script>

<style scoped>
span.subtle {
    color: gray;
}

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
    margin: 0 auto;
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

    border: gray dashed 1px;
    border-radius: 0.15em;
    text-align: center;
}

.game-grid .cell.factory {
    background: lightblue;
}

.game-grid .cell.select {
    background: #bbb;
}

.game-grid .cell.plus {
    background-image: url(/sprites/cell-plus.png);
    background-clip: content-box;
    background-position: center;
    background-size: cover;
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

.item-list {
    margin: 0 auto 2em auto;
    display: flex;
    width: auto;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
}

.item-list .item {
    /* TODO: responsive padding */
    padding:6px;
    border: gray dashed 1px;
    border-radius: 0.15em;
}

.item-list .item.highlight {
    background: #eb983c;
}

.item-list .item img {
    width: 64px;
    aspect-ratio: 1;
}

.item-list .item .overlay {
    position: relative;
}

.item-list .item .overlay div {
    font-weight: 900;
    font-size: 0.9em;
    -webkit-text-stroke-width: 1px;
    -webkit-text-stroke-color: white;
}

.item-list .item .overlay .sw {
    position: absolute;
    left: 0;
    bottom: -5px;
}

.simple-table {
    width: 100%;
}

.simple-table tr {
    border-bottom: 1px solid lightgray;
}

.simple-table tr:first-child {
    border-top: 1px solid lightgray;
}

.simple-table tr td {
    width: 50%;
    padding: 0.2em 0.5em;
    text-align: left;
}

.simple-table tr td:first-child {
    font-weight: bold;
    text-align: right;
}
</style>

<style>
body {
    background: #f6eada;
}

.game-grid {
    background: #dab382;
}

/* Patch to fix inactive mode button staying highlighted on mobile */
.tabs .btn-outline-dark:not(.disabled):not(.active):hover,
.tabs .btn-outline-dark:not(.disabled):not(.active):active {
    background: transparent !important;
    color: #343a40 !important;
    border-color: #343a40 !important;
}
</style>
