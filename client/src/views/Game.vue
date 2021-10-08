<template>
  <div>
    <loader v-if="!app.ready || !app.game || !app.game.ready" />

    <div v-else class="text-center">
        <div class="game">

            <!-- Pause indicator -->
            <b-alert v-if="!app.running" show variant="dark">
                <img src="/sprites/pause-button.png"
                    class="pause-icon blink"
                    title="Paused"
                    alt="Paused"
                    draggable="false"
                />
                Game paused
            </b-alert>

            <!-- Stats header -->
            <div class="header">
                <h1 class="h3 fw-normal title">
                    <span v-if="$auth.session">{{ $auth.session.name }}</span>
                    <span v-else>Game</span>
                </h1>

                <h5 class="h5 fw-normal left">
                    Money:
                    {{ app.game.inventory.money }}
                </h5>
                <h5 class="h5 fw-normal right">
                    Energy:
                    {{ app.game.inventory.energy }}
                </h5>
            </div>

            <!-- Toolbar -->
            <b-button-group class="toolbar tabs w-100" size="lg">
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
                    @click.prevent="showBuy()"
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
                <div v-for="(cell, index) in app.game.inventory.items"
                    class="cell"
                    @click.stop="toggleSelect(index)"
                    v-bind:class="{ select: selected == index, item: cell, factory: cell && cell.drop_interval, subtle: isSubtle(index), plus: !cell && mode == 'buy' && buyItem, odd: isOdd(index) }"
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

            <!-- Action buttons -->
            <b-button-group class="toolbar w-100" size="lg">
                <b-button
                    type="button"
                    class="w-100"
                    variant="outline-dark"
                    @click.stop.prevent="actionScanCode"
                    squared
                    :disabled="!app.running"
                >Scan code</b-button>
                <b-button
                    v-if="$auth.auth && $auth.hasRoleAdmin()"
                    type="button"
                    class="w-100"
                    variant="outline-danger"
                    @click.stop.prevent="actionMockScanCode"
                    squared
                    :disabled="!app.running"
                >Mock scan code</b-button>
            </b-button-group>
        </div>

        <!-- Buy modal -->
        <b-modal
            id="game-buy-modal"
            title="Buy item"
            centered
            no-fade
        >
            <div class="buy-list">
                <div
                    v-for="item in app.game.getBuyableItems()"
                    class="entry"
                    @click.stop.prevent="selectBuyItem(item)"
                >
                    <div class="item">
                        <img :src="'/sprites/' + item.sprite"
                            :title="item.name"
                            :alt="item.name"
                            draggable="false"
                        />
                    </div>

                    <div class="details">
                        <h1>{{ item.name }}</h1>

                        Costs:
                        <ul>
                            <li v-if="item.buy" v-for="amount in Object.values(item.buy)">
                                <span v-if="amount.money">{{ amount.money }} money</span>
                                <span v-if="amount.energy">{{ amount.energy }} energy</span>
                                <span v-if="amount.item">
                                    <span v-if="amount.quantity > 1">{{ amount.quantity }}×</span>
                                    <img :src="'/sprites/' + app.game.items[amount.item].sprite"
                                        :title="app.game.items[amount.item].name"
                                        :alt="app.game.items[amount.item].name"
                                        draggable="false"
                                        class="item tiny"
                                    />
                                    {{ app.game.items[amount.item].name }}
                                </span>
                            </li>
                        </ul>
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
                <div class="tier-list">
                    <div
                        v-for="(item, index) in app.game.getDownUpgradeItems(selectedCell.ref)"
                        class="item"
                        v-bind:class="{ highlight: item.ref == selectedCell.ref }"
                    >
                        <img v-if="selectedCell.ref == item.ref || app.game.isDiscovered(item.ref)"
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

                <p v-if="app.game.items[selectedCell.ref].description" class="font-italic">
                    {{ app.game.items[selectedCell.ref].description }}
                </p>

                <table class="simple-table">
                    <tr><td>Name:</td><td>{{ selectedCell.name }}</td></tr>
                    <tr><td>Tier:</td><td>{{ selectedCell.tier }}</td></tr>
                    <tr v-if="selectedCell.label"><td>Label:</td><td>{{ selectedCell.label }}</td></tr>
                    <tr v-if="selectedCell.drop_interval"><td>Production speed:</td><td><span class="subtle">1 / </span>{{ selectedCell.drop_interval }} ticks</td></tr>
                    <tr v-if="selectedCell.drop_limit"><td>Drops:</td><td>{{ selectedCell.drop_limit}} <span class="subtle"> times</span></td></tr>
                    <tr v-if="selectedCell.drop_interval">
                        <td>Drops:</td>
                        <td>
                            <ul class="drops-list">
                                <li v-for="drop in app.game.items[selectedCell.ref].drops">
                                    <span v-if="selectedCell.ref == drop.item || app.game.isDiscovered(drop.item)">
                                        <img :src="'/sprites/' + app.game.items[drop.item].sprite"
                                            :title="app.game.items[drop.item].name"
                                            :alt="app.game.items[drop.item].name"
                                            draggable="false"
                                            class="item tiny"
                                        />
                                        {{ app.game.items[drop.item].name }}
                                    </span>
                                    <span v-else>
                                        <img src="/sprites/white-question-mark.png"
                                            title="Undiscovered item"
                                            alt="Undiscovered item"
                                            draggable="false"
                                            class="item tiny"
                                        />
                                        ?
                                    </span>
                                    <span class="subtle">{{ formatPercentage(drop.chance) }}</span>
                                </li>
                            </ul>
                        </td>
                    </tr>
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
import utilFormat from "../util/format.js";

export default {
  name: "Game",
  data() {
    return {
      app: this.$app,
      mode: null,
      selected: null,
      selectedCell: null,
      buyItem: null,
    };
  },
  created() {
    // Check auth, initialize game or redirect to login
    this.$auth
        .isAuth()
        .then((auth) => {
            // User must have game role
            if(this.$auth.hasRoleGame()) {
                this.$app.init(this);
                this.$app.initGame();
            } else
                this.redirectToLogin();
        });
  },
  methods: {
    redirectToLogin() {
        this.$router.push({name: "login"});
    },

    showScanner() {
        this.$router.push({name: "scan"});
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
        // When selecting second cell, merge or swap
        if(this.selected !== null && index !== null && this.selected !== index) {
            switch(this.mode) {
                // Merge
                case 'merge':
                    if(this.canMerge(index)) {
                        this.actionMerge(this.selected, index);
                        return;
                    }
                    break;

                // Swap
                case null:
                    if(this.hasItem(this.selected)) {
                        this.actionSwap(this.selected, index);
                        return;
                    }

                default:
            }
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

    actionSwap(index, otherIndex) {
        // First index must be an item
        if(!this.hasItem(index))
            return;

        console.debug("[game] Swapping items");

        // Send merge action
        this.app.socket.send('action_swap', {
            cell: index,
            other: otherIndex,
        });

        // Premove swap on client, clear selection
        this.app.game.premoveSwap(index, otherIndex);
        this.selected = null;
    },

    actionMerge(index, otherIndex) {
        // We must merge two items
        // TODO: do not allow to merge items of different types
        if(!this.hasItem(index) || !this.hasItem(otherIndex))
            return;

        console.debug("[game] Merging items");

        // Send merge action
        this.app.socket.send('action_merge', {
            cell: otherIndex,
            other: index,
        });

        // Premove remove and merge on client, clear selection
        this.app.game.premoveMerge(otherIndex, index);
        this.selected = null;
    },

    // Show buy item selection dialog.
    showBuy() {
        // We'll enable buy mode again after selecting a buy item
        this.mode = null;

        // Show buy modal
        this.$bvModal.show('game-buy-modal');
    },

    // Select an buy item.
    selectBuyItem(item) {
        // Hide dialog
        this.$bvModal.hide('game-buy-modal');

        this.mode = 'buy';
        this.buyItem = item;
    },

    actionBuy(index) {
        // Cell must be empty
        if(this.hasItem(index))
            return;

        console.debug("[game] Buying item");

        // Send buy action
        let ref = this.buyItem.ref;
        this.app.socket.send('action_buy', {
            cell: index,
            item: ref,
        });

        // Reset selection, premove placement for instant feedback
        this.app.game.premovePlace(index, ref);
        this.selected = null;
    },

    actionSell(index) {
        // Cell must not be empty
        if(!this.hasItem(index))
            return;

        console.debug("[game] Selling item");

        // Send sell action
        this.app.socket.send('action_sell', {
            cell: index,
        });

        // Reset selection, clear cell for instant feedback
        this.app.game.premoveRemove(index);
        this.selected = null;
    },

    actionDetails(index) {
        // Cell must not be empty
        if(!this.hasItem(index))
            return;

        console.debug("[game] Showing item details");

        // Show details modal
        this.selectedCell = this.app.game.inventory.items[index];
        this.$bvModal.show('game-details-modal');
    },

    actionScanCode() {
        console.debug("[game] Scanning code");
        this.showScanner();
    },

    actionMockScanCode() {
        console.debug("[game] Mock scan code");

        // Send scan code action
        this.app.socket.send('mock_scan_code', null);
    },

    /**
     * Whether a cell is odd.
     */
    isOdd(index) {
        return ((index % 16) < 8 && (index % 2) == 0)
            || ((index % 16) >= 8 && (index % 2) == 1);
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
                if(!this.app.game.inventory.items[index].mergeable)
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
        return index !== null && !!this.app.game.inventory.items[index];
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

        let a = this.app.game.inventory.items[other];
        let b = this.app.game.inventory.items[index];

        // Must be mergeable, must have same ID
        return a.mergeable && a.ref == b.ref;
    },

    formatPercentage(value) {
        return utilFormat.formatPercentage(value);
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

.pause-icon {
    width: 1em;
    height: 1em;
    margin: 0 0.2em 0 0;
    position: relative;
    top: -1px;
}

.blink {
    animation: blink-animation 1s steps(2, start) infinite;
}

@keyframes blink-animation {
    to {
        visibility: hidden;
    }
}

.game {
    margin: 0 auto;
    max-width: 70vh;
}

.header {
    display: grid;
    grid-template-areas: "left title right";
    gap: 0.2em 1em;
    justify-items: stretch;
    align-items: center;
    margin: 0 0 1em 0;
}

@media screen and (max-width: 470px) {
    .header {
        grid-template-areas:
            "title title"
            "left right";
    }
}

.header .title {
    grid-area: title;
    margin: 0;
}

.header .left {
    grid-area: left;
    text-align: left;
    margin: 0;
}

.header .right {
    grid-area: right;
    text-align: right;
    margin: 0;
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
    box-sizing: border-box;
    background: #9d9696;

    /* Disable selecting cells. */
    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -o-user-select: none;
    user-select: none;

    /* Disable double-tap to zoom on iPhone */
    touch-action: manipulation;
}

.game-grid .cell {
    display: inline-block;
    aspect-ratio: 1;
    padding: var(--grid-space);
    box-sizing: content-box;

    border-radius: 0.15em;
    text-align: center;
    cursor: pointer;

    background-image: url("/sprites/tile-light.png");
    background-clip: padding-box;
    background-position: center;
    background-repeat: no-repeat;
    background-size: cover;
}

.game-grid .cell.odd {
    background-image: url("/sprites/tile-dark.png");
}

.game-grid .cell.factory {
    background-image: url("/sprites/tile-light-factory.png");
}

.game-grid .cell.factory.odd {
    background-image: url("/sprites/tile-dark-factory.png");
}

.game-grid .cell.select {
    background: #bbb;
}

.game-grid .cell.plus {
    background-image: url(/sprites/cell-plus.png);
    background-position: center;
    background-size: cover;
}

.game-grid .cell.select.item {
    background: #eb983c;
}

.game-grid .cell.subtle {
    opacity: 0.3;
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

.tier-list {
    margin: 0 auto 2em auto;
    display: flex;
    width: auto;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
}

.tier-list .item {
    /* TODO: responsive padding */
    padding:6px;
    border: gray dashed 1px;
    border-radius: 0.15em;
}

.tier-list .item.highlight {
    background: #eb983c;
}

.tier-list .item img {
    width: 64px;
    aspect-ratio: 1;
}

.tier-list .item .overlay {
    position: relative;
}

.tier-list .item .overlay div {
    font-weight: 900;
    font-size: 0.9em;
    -webkit-text-stroke-width: 1px;
    -webkit-text-stroke-color: white;
}

.tier-list .item .overlay .sw {
    position: absolute;
    left: 0;
    bottom: -5px;
}

.buy-list {
    display: flex;
    width: 100%;
    flex-direction: column;
    align-items: flex-start;
    align-content: flex-start;
    gap: 10px;
}

.buy-list .entry {
    padding:6px;
    border: gray dashed 1px;
    border-radius: 0.15em;
    display: flex;
    align-content: stretch;
    gap: 10px;
    width: 100%;
    cursor: pointer;
}

.buy-list .entry > .item {
    /* TODO: responsive padding */
    padding:6px;
    /* border: gray dashed 1px; */
    border-radius: 0.15em;
}

.buy-list .entry > .item img {
    width: 64px;
    aspect-ratio: 1;
}

.buy-list .entry .details {
    flex-grow: 1;
    font-size: 0.8em;
}

.buy-list .entry .details h1 {
    margin: 0 0 0.35em 0;
    font-weight: bold;
    font-size: 1rem;
}

.buy-list .entry .details ul {
    margin: 0;
    padding: 0 0 0 1.5em;
}

ul.drops-list {
    list-style: none;
    margin: 0;
    padding: 0;
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

img.item.tiny {
    width: 1.35em;
    height: 1.35em;
    display: inline;
    text-align: text-top;
    border: gray dashed 1px;
    border-radius: 0.15em;
    padding: 0.05em;
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
.toolbar.tabs .btn-outline-dark:not(.disabled):not(.active):hover,
.toolbar.tabs .btn-outline-dark:not(.disabled):not(.active):active {
    background: transparent !important;
    color: #343a40 !important;
    border-color: #343a40 !important;
}
</style>
