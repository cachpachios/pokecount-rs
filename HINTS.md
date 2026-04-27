# Hints

Read these in order. Each one gives a bit more away. Try to stop reading as soon as you've got the next idea.

## Hint 1: try the obvious thing first

Wrap the loop body in `tokio::spawn(async move { ... })`, collect the handles in a `Vec`, then `.await` them at the end.

Do `cargo run`. The compiler will yell at you.

## Hint 2: why it's yelling

Two reasons, probably:

1. `tokio::spawn` needs the future to be `'static`. Its abit cryptic but its just that we can't hold references to stuff on `main`'s stack in the future since the future can outlive it. Your `&mut client` lives on that stack.
2. Even if you tried to move `client` in, **only one task can own it**. Rust is not gonna let you `&mut` the same thing from multiple places.

So you need shared ownership of the client, and the cache inside it needs to be safely mutable from many tasks.

## Hint 3: the two tools

- `Arc<T>` is shared ownership. Clone on an Arc is cheap, just bumps an atomic refcount. Now multiple tasks can each hold their own `Arc` pointing at the same thing.

- `Mutex<T>` wraps data and only lets one thread touch it at a time. `lock()` gives you a guard, drop the guard to release. The data lives _inside_ the mutex, so you literally can't access it without locking.

## Hint 4: where to put the Arc/Mutex

You've got two flavors:

a) Wrap everything: `Arc<Mutex<PokeCachedMoveClient>>`. Simple, but you hold the lock across HTTP calls. That serializes everything again. Whoops.

b) Wrap the PokeCachedMoveClient in an `Arc`. Now you get another issue, you not allowed to do mutating calls. But we (obviously) mutate the internals when populating the cache.

Option (b) is what you want.
But you should wrap the `move_cache: Mutex<HashMap<String, Move>>` then you can drop the `mut` from `fetch_move(&mut self, ...)` and suddenly the compiler will be happy. (after you add the .lock() ofc...)

## Hint 5: joining the handles

`tokio::spawn` returns a `JoinHandle<T>`. The task starts running the moment you spawn it, you don't have to await it for it to make progress. Awaiting just gives you the result back.

So the pattern is: spawn everything in the loop, push the handles into a `Vec`, then in a second loop await them one by one to collect results. By the time you start awaiting, most of them are probably done already.

```rust
let mut handles = Vec::new();
for entry in list.results {
    let client = Arc::clone(&client);
    handles.push(tokio::spawn(async move {
        process_pokemon(&client, entry.url).await
    }));
}

let mut results = HashMap::new();
for h in handles {
    let (name, moves) = h.await??; // outer ? for JoinError, inner ? for app error
    results.insert(name, moves);
}
```

## Bonus: the sneaky race

Your cache does check-then-insert. If two tasks ask for the same move URL at the same time, both will see "not in cache", both will fetch, both will insert. You did two HTTP calls instead of one.

Is this a bug? Eh. The data is the same, last write wins.
Noteworth is that Rust prevented the _data race_ but not the _logic race_. Bugs is a thing in Rust. Just not as common ;)

If you really wanted to fix it, you'd need something like a per-key `OnceCell` or a "lock the slot, fetch under the lock" pattern. Out of scope for today, but worth knowing the difference.
