import ws from "./ws.js";

// Poll inventory state every five minutes.
const INVENTORY_POLL_INTERVAL = 5 * 60 * 1000;

export default {
    // Game readyness state.
    //
    // Changes to true when game state is received from server.
    ready: false,

    // App state.
    app: null,

    // Current inventory.
    inventory: null,

    // Timer to poll latest inventory state.
    inventory_poll_timer: null,

    // Game configuration items.
    items: null,

    // Initialize.
    init(app) {
        console.log('[game] Initializing...');

        this.app = app;
        this.ready = false;

        // Register message listeners
        this.app.socket.addListener('inventory', (data) => this.onMsgInventory(data));
        this.app.socket.addListener('inventory_balances', (data) => this.onMsgInventoryBalances(data));
        this.app.socket.addListener('inventory_cell', (data) => this.onMsgInventoryCell(data));
        this.app.socket.addListener('inventory_discovered', (data) => this.onMsgInventoryDiscovered(data));
        this.app.socket.addListener('config_items', (data) => this.onMsgConfigItems(data));

        // Ask server for game state
        this.pollGameState();

        // Set up new timer to periodically poll full inventory state
        if(this.inventory_poll_timer !== null)
            clearInterval(this.inventory_poll_timer);
        this.inventory_poll_timer = setInterval(
            () => this.pollInventoryState(),
            INVENTORY_POLL_INTERVAL,
        );
    },

    // Set inventory cell item.
    setCell(index, item) {
        this.inventory.items.splice(index, 1, item);
    },

    // Check wheher the user discovered an item.
    isDiscovered(ref) {
        return this.inventory.discovered.includes(ref);
    },

    // Get a list of items the user may buy.
    //
    // Returns array of items configurations.
    getBuyableItems() {
        return Object.values(this.items)
            .filter(i => i.buy !== null && i.buy !== undefined)
            .sort((a, b) => this.items[a.ref].client_order - this.items[b.ref].client_order);
    },

    /// Find the item before the given reference.
    //
    // Returns null or the item configuration.
    getDowngradeItem(ref) {
        return Object.values(this.items)
            .find(i => i.merge == ref) || null;
    },

    /// Get a list of downgrade and upgrade items for the given item reference.
    //
    // Returns array of item configurations, including itself.
    getDownUpgradeItems(ref) {
        return this.getDowngradeItems(ref)
            .concat([this.items[ref]])
            .concat(this.getUpgradeItems(ref));
    },

    // Get list of items, in order, this is upgraded from.
    //
    // Returns array of item configuration, excluding itself.
    getDowngradeItems(ref) {
        let down_item = this.getDowngradeItem(ref);
        if(down_item == null)
            return [];

        let before = this.getDowngradeItems(down_item.ref);

        // Return before and down item, but prevent infinite loops
        if(!before.includes(down_item))
            return before.concat([down_item]);
        return before;
    },

    // Get list of items, in order, this upgrades to.
    //
    // Returns array of item configurations, excluding itself.
    getUpgradeItems(ref) {
        let item = this.items[ref];
        if(item.merge === null)
            return [];

        let merge_item = this.items[item.merge];
        let after = this.getUpgradeItems(item.merge);

        // Return merge item and after, but prevent infinite loops
        if(!after.includes(merge_item))
            return [merge_item].concat(after);
        return after;
    },

    // Poll latest game state from server.
    pollGameState() {
        this.app.socket.send('get_game', null);
    },

    // Poll latest inventory state from server.
    pollInventoryState() {
        this.app.socket.send('get_inventory', null);
    },

    // Premove the item at the given index, remove it.
    //
    // Game state may be come inconsistent, but we should receive the game state
    // from the server shortly which will fix this.
    premoveRemove(index) {
        this.inventory.items[index] = null;
    },

    // Premove the items at the given indices, merge other into index.
    //
    // Game state may be come inconsistent, but we should receive the game state
    // from the server shortly which will fix this.
    premoveMerge(index, otherIndex) {
        this.premoveUpgrade(index);
        this.premoveRemove(otherIndex);
    },

    /// Remove the item at the given index, place item with given reference.
    premovePlace(index, ref) {
        // Get the item
        let item = this.items[ref];
        if(item === null || item === undefined)
            return;

        // Instantiate item
        this.setCell(index, {
            ref: item.ref,
            name: item.name,
            tier: item.tier,
            label: item.label,
            sell: item.sell,
            drop_interval: item.drop_interval,
            drop_limit: item.drop_limit,
            sprite: item.sprite,
            mergeable: item.merge !== null,
        });
    },

    // Premove the item at the given index, upgrade it a level.
    //
    // Game state may be come inconsistent, but we should receive the game state
    // from the server shortly which will fix this.
    premoveUpgrade(index) {
        // Get the upgrade item
        let ref = this.items[this.inventory.items[index].ref].merge;
        let item = this.items[ref];
        if(item === null || item === undefined)
            return;

        // Place upgraded item, set temporary sync label
        this.inventory.items[index].ref = item.ref;
        this.inventory.items[index].name = item.name;
        this.inventory.items[index].tier = item.tier;
        this.inventory.items[index].label = item.label;
        this.inventory.items[index].sprite = item.sprite;
    },

    // Premove the items at the given indices, swap them.
    //
    // Game state may be come inconsistent, but we should receive the game state
    // from the server shortly which will fix this.
    premoveSwap(index, otherIndex) {
        let tmp = this.inventory.items[index];
        this.inventory.items[index] = this.inventory.items[otherIndex];
        this.inventory.items[otherIndex] = tmp;
    },

    /**
     * Handle inventory message from server.
     */
    onMsgInventory(inventory) {
        this.inventory = inventory;
        this.ready = true;
    },

    /**
     * Handle inventory balances message from server.
     */
    onMsgInventoryBalances(data) {
        this.inventory.money = data.money;
        this.inventory.energy = data.energy;
    },

    /**
     * Handle inventory cell message from server.
     */
    onMsgInventoryCell(data) {
        this.setCell(data.index, data.item);
    },

    /**
     * Handle inventory discovered message from server.
     */
    onMsgInventoryDiscovered(discovered) {
        this.inventory.discovered = discovered;
    },

    /**
     * Handle config items message from server.
     */
    onMsgConfigItems(items) {
        this.items = items;
    },
};
