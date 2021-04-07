# spelling
A spellchecker written in rust.
# How to use
```rust
use spelling::spellcheck;

fn main() {
    let dictionary_string = include_str!("words.txt"); // newline separated 
    spellcheck(dictionary_string, "restaraunt", 3);
}
```

# Details
This uses the [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance) as the heuristic for distance.
