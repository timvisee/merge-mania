import ws from "./ws.js";

export default {
    // Game readyness state.
    //
    // Changes to true when game state is received from server.
    ready: false,

    // Current inventory.
    inventory: null,

    // Game configuration items.
    items: null,

    // Game socket manager.
    socket: null,

    // Initialize.
    init(vueContext) {
        this.ready = false;

        // TODO: construct fresh socket!
        this.socket = ws;
        this.socket.vueContext = vueContext;
        this.socket.connect(this);
    },

    // Check wheher the user discovered an item.
    // TODO: implement this using discovered index
    // TODO: remove currentRef
    isDiscovered(ref, currentRef) {
        return parseInt(ref.split('.')[1]) <= parseInt(currentRef.split('.')[1]);
    },

    // Get a list of items the user may buy.
    //
    // Returns array of items configurations.
    getBuyableItems() {
        return Object.values(this.items)
            .filter(i => i.buy !== null && i.buy !== undefined)
            .sort((a, b) => parseInt(a.ref.split('.')[0]) - parseInt(b.ref.split('.')[0]));
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

    // Premove the item at the given index, upgrade it a level.
    //
    // Game state may be come inconsistent, but we should receive the game state
    // from the server shortly which will fix this.
    premoveUpgrade(index) {
        // Get the upgrade item
        let ref = this.items[this.inventory.items[index].ref].merge;
        if(ref === null || ref === undefined)
            return;
        let item = this.items[ref];
        if(item === null || item === undefined)
            return;

        // Set upgraded values
        this.inventory.items[index].ref = item.ref;
        this.inventory.items[index].name = item.name;
        this.inventory.items[index].tier = item.tier;
        this.inventory.items[index].label = 'sync';
        // TODO: use this instead: this.inventory.items[index].label = item.label;
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
};
