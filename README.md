# Pokemon Move counter - Rusty edition

Tiny Rust program that hits the [PokéAPI](https://pokeapi.co/), grabs a list of Pokémon, then for each one fetches its moves and groups them by type. There's a little cache on the client so we don't refetch the same move.

## The assignment

Make it concurrent.

Look in `src/main.rs` for the `TODO`. The loop in `main` processes Pokémon one by one. Your job is to spawn them all at once with `tokio::spawn` and join the handles at the end.

## Running it

If you are missing Rust/Cargo: [https://rustup.rs/](https://rustup.rs/)

```sh
cargo run
```

Or with a custom limit:

```sh
cargo run -- 50
```

## What "done" looks like here

- It compiles (the hard part).
- Output looks the same as before (same Pokémon, same move counts).
- Wall clock dropped a lot.

If you get stuck, peek at `HINTS.md`.
