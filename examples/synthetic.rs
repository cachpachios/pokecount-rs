use std::collections::HashMap;
use std::time::{Duration, Instant};

struct Cache {
    data: HashMap<u32, u32>,
}

impl Cache {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    async fn get(&mut self, key: u32) -> u32 {
        if let Some(v) = self.data.get(&key) {
            return *v;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        let v = key * 2;
        self.data.insert(key, v);
        v
    }
}

#[tokio::main]
async fn main() {
    let clock = Instant::now();
    let mut cache = Cache::new();

    let keys = vec![1, 2, 3, 4, 1, 2, 5, 6];

    // TODO: make this concurrent with tokio::spawn, joining the handles at the end.
    for k in keys {
        let v = cache.get(k).await;
        println!("{k} -> {v}");
    }

    println!("Done in {:.2?}", clock.elapsed());
}
