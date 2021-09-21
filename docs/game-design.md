# Game design

## Elements
- teams
  - mobile page
  - factories
    - must be bought
    - can be upgraded (better drop chances)
    - produces items in inventory at time interval
    - item queue if no space
  - inventory (size limit) (grid?)
    - money
    - items (type, level)
- trading posts (QR)
  - gives you energy (credits)
    - merge/sell/build/upgrade costs energy
- QR code page
- admin page
  - start/stop game
  - manage users
  - teams
  - balance
  - inventories

## Design

Sprites: 16x16 PNG

Product types:
- Fruit:
  - Apple (cost 1)
  - Pear (cost 2)
  - Banana
  - Ananas
- Vegetable (tier/product):
  - Tomato
  - Bell peppert
- Mushroom (tier/product):
- Eggs:
  - Dragon egg
- Drinks:
- Wheat

Factories:
- Fruit:
  - Level 1:
    - Buy cost: 5 money
    - Sell cost: 2 money
    - production time: 1 min
    - 95% apple, 4% Pear, 1% Tomato
  - Level 2:
    - Buy cost: 20 money
    - Sell cost: 5 money
    - production time: 45 sec
    - 87% apple, 10% Pear, 3% Tomato
  - Level 3: ...
- Vegetable:
  - Build cost: cost 50, Banana, Ananas
  - Level 1:
    - production time: 45 sec
    - 96% Tomato, 4% Bell pepper
  - Level 2: ...
  - Level 3: ...

Energy requirements:
- building factories
- merging products
- merging factories
- selling items

Initial inventory items (default items, factories)

## Future ideas

- effort vs reward -> more qr code credits
  - scan all QRs instead of just 2 for more credits
- increase factory price when buying more
- tag each other, scan others QR code: trade/steal
- missions
  - gather items to complete missions
  - production time factor
