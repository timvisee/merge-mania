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
    init() {
        this.ready = false;

        // TODO: construct fresh socket!
        this.socket = ws;
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
            .filter(i => i.buy !== null && i.buy !== undefined);
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
};
