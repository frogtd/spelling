# spelling
A spellchecker written in rust.
# How to use
```rust
use spelling::spellcheck;

let dictionary_string = include_str!("words.txt"); // newline separated 
spellcheck(dictionary_string, "restaraunt", 3);
```
If you can't use rayon use `default-features = false` in your Cargo.toml.
```toml
[dependencies]
spelling = { version = "2.2", default-features = false }
```
# Details
This uses the [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
as the heuristic for distance.