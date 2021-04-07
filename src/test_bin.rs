use spelling::spellcheck;

fn main() {
    let dictionary_string = include_str!("words.txt");
    for _ in 0..100 {
        spellcheck(dictionary_string, "restaraunt", 3);
    }
}