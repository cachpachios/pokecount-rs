use std::collections::HashMap;
use std::time::Instant;

use serde::Deserialize;

const BASE_URL: &str = "https://pokeapi.co/api/v2";
const DEFAULT_LIMIT: usize = 20;

#[derive(Deserialize)]
struct PokemonList {
    results: Vec<NamedRef>,
}

#[derive(Deserialize)]
struct NamedRef {
    url: String,
}

#[derive(Deserialize)]
struct Pokemon {
    name: String,
    moves: Vec<MoveEntry>,
}

#[derive(Deserialize)]
struct MoveEntry {
    #[serde(rename = "move")]
    move_ref: NamedRef,
}

#[derive(Deserialize, Clone)]
struct Move {
    name: String,
    #[serde(rename = "type")]
    move_type: TypeRef,
}

#[derive(Deserialize, Clone)]
struct TypeRef {
    name: String,
}

struct PokeCachedMoveClient {
    http_client: reqwest::Client,
    move_cache: HashMap<String, Move>,
}

impl PokeCachedMoveClient {
    fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            move_cache: HashMap::new(),
        }
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        Ok(self.http_client.get(url).send().await?.json::<T>().await?)
    }

    async fn fetch_move(&mut self, url: &str) -> anyhow::Result<Move> {
        if let Some(m) = self.move_cache.get(url) {
            return Ok(m.clone());
        }
        let m: Move = self.get(url).await?;
        self.move_cache.insert(url.to_string(), m.clone());
        Ok(m)
    }

    fn num_cached_moves(&self) -> usize {
        self.move_cache.len()
    }
}

async fn process_pokemon(
    client: &mut PokeCachedMoveClient,
    url: String,
) -> anyhow::Result<(String, HashMap<String, Vec<String>>)> {
    let pokemon: Pokemon = client.get(&url).await?;
    let mut by_type: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &pokemon.moves {
        let m = client.fetch_move(&entry.move_ref.url).await?;
        by_type.entry(m.move_type.name).or_default().push(m.name);
    }
    println!("  done: {} ({} moves)", pokemon.name, pokemon.moves.len());
    Ok((pokemon.name, by_type))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let limit: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_LIMIT);

    let clock = Instant::now();
    let mut client = PokeCachedMoveClient::new();

    let list: PokemonList = client
        .get(&format!("{BASE_URL}/pokemon?limit={limit}"))
        .await?;
    println!("Processing {} Pokémon...", list.results.len());

    // TODO: this loop is sequential. Make it concurrent with tokio::spawn, joining the handles at the end. Be careful with the client and its cache!
    let mut results = HashMap::new();
    for entry in list.results {
        let (name, moves) = process_pokemon(&mut client, entry.url).await?;
        results.insert(name, moves);
    }

    println!(
        "Done in {:.2?} — {} pokémon, {} moves cached",
        clock.elapsed(),
        results.len(),
        client.num_cached_moves(),
    );
    Ok(())
}
