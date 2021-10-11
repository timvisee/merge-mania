# Merge Mania game

A real-time team based merging/trading game to play using your phone.

- Played in teams for a set time period
- Players, in a team, use their phone to manage their game
- Each team has _money_, _energy_ and an inventory grid
- Factory items, on your grid, produce items at a set interval
- **Merge** items of the same type into better items
- Discover new items and tiers as you play
- Players run between outposts to earn _energy_
- _Energy_ is used to buy additional _factories_ on a grid, producing new items
- Buy lots of factories to gain items as quickly as possible
- Merge & sell smart to earn as much as possible
- After playing, the team earning the most wins

## Player explanation

This game is played using your mobile phone through a website.
You play in teams, consisting of any number of players.
Your goal is to earn as much money as possible within a set time frame.

Each team has money, energy, and an inventory in the form of an item grid.

You may merge items of the same type into one better item. It's almost always a
good idea to merge rather than sell. This will free up an inventory cell, and
makes your inventory better.
You may also buy or sell new items for a set price.

Light blue cells are _factory_ items, producing items in your inventory grid at
a set interval. Buy and merge factories to improve your production rate and
value. When a factory is merged, it'll instantly produce a new item, skipping
the production time.

As a player you may walk around between different outposts, set up by the game
administrator. At these outposts you may scan a QR-code to gain _energy_ and
_money_. This is required to buy as much factories as possible. You can't scan
the same outpost twice, without going to another first.

Tip: start with a lot of basic factories (trees), without merging them, to
produce as much items as possible. When you can't handle the production rate,
start merging them into better factories, having a lower shared production rate,
but dropping better items.

Tip: scanning more unique outposts after each other means gaining more _energy_
and _money_.

## How to use

This repository contains the files you'll need to set up this game, including:

- Server software
- Client software
- Configuration file

The server acts as a web server, which is what clients talk to. The client
software is delivered to devices when connecting.

This is a very basic guide on how to set things up.

### Requirements

- A machine to host the game
- Tools:
  - `git`
  - [Rust](https://rustup.rs/)
  - [`node`](https://nodejs.org/en/download/) / `npm`
- To play online: a domain with TLS certificate

### Prepare

Clone the repository to your machine:

```bash
git clone https://gitlab.com/timvisee/merge-mania
cd merge-mania
```

### Set up server

After you've cloned the repository and installed Rust, change into the `server/`
directory. Use `cargo` to build a release version of the server software.

```bash
# Change into ./server
cd server

# Build release software
cargo build --release
```

Configure logging using an `.env` file:

```bash
# Use sample .env
cp .env.sample .env

# Change .env if desired, you may keep this as-is
```

Run the server through `cargo`, invoke the binary directly, or install the
server on your system.

```bash
# Run through cargo
cargo run --release

# Or, run binary directly
./target/release/mms

# Or, install on your system and run it
cargo install --release
mms
```

This starts the server on port 8000.

### Set up client

The client must be compiled to get a distributable version that can be served to
clients by the server.

After you've installed `node` and `npm`, change into the `./client` directory.
Use `npm` to build the client software:

```bash
# Change into ./client
cd client

# Build distributable client
npm run build
```

### Configuration

When the server and client are ready to be used, it's time to set up the game
configuration.

Open the `config/config.toml` file, and configure it to your liking.

- You must change the `outposts.secret` value to something random, keep it
  secret.
- Configure the `users` list to a set of desired teams.
  Users having `role_game = true` can play the game,
  users having `role_admin = true` can manage and reset the game.
- A lot of game `items` have been preconfigured, tweak this if desired.

After configuring, save the file, and restart the server.

### Playing locally

The server currently starts listening on port 8000.

You may now play the game locally, on the same machine running the server.
Navigate to `http://localhost:8000/`

### Playing online

To play online with others, you must use a public facing domain. TLS (https) is
required to allow usage of the devices camera for scanning outpost QR-codes.

The Merge Mania server has no option to configure TLS certificate, thus this
must be done through a reverse proxy.

Set up a reverse proxy with a public facing domain, and configure TLS on it.
See [`./docs/nginx.conf`](./docs/nginx.conf) as example site for Nginx.

### Playing the game

When the server is properly running and reachable, navigate to it in your
browser. Log in with an admin account (`role_admin = true`) and navigate to the
'Admin' page. Manage the game from here, and create outposts before starting the
game.

## License
This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.
