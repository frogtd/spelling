#![feature(test)]


use rayon::prelude::*;



/// Takes a `dictionary_string` (newline separated), a word and a distance and
/// returns a vector of possible matches, with a limit of distance set up
/// `distance`.
/// Note: this only works for single length code bytes
/// How it works:
///
/// It loops through each word in the dictionary and then you have a word from
/// the dictionary and a word to match against.
///
/// Lets say those words were `rumor` and `rotten`. You then create a vector
/// that is the length of the shorter word + 1, counting up.
/// ```
/// let mut list = vec![0, 1, 2, 3, 4, 5];
/// ```
/// You also keep track a left variable, which starts at the index of the
/// longest string you're at.
/// ```
/// # let longest_word = "rotten";
/// for x in 2..longest_word.len() {
///     let mut left = x;
/// }
/// ```
///
pub fn spellcheck<'a>(dictionary_string: &'a str, word: &str, distance: usize) -> Vec<&'a str>  {
    let mut vec: Vec<_> = dictionary_string
        .split('\n')
        .collect::<Vec<&str>>()
        .par_iter()
        .map(|string_in| {

            use std::ops::Range;
            let (shorter, longer) = {
                if string_in.len() > word.len() {
                    (*string_in, word)
                } else {
                    (word, *string_in)
                }
            };

            let mut list: Vec<usize> = vec![1];
            for index in 1..shorter.len() {
                list.push(index)
            }
            .collect();

            for x in 2..(longer.len() + 1) {
                let mut left = x;
                let mut temp: Vec<usize> = Vec::with_capacity(shorter.len()+1);
                temp.push(left);
                let mut iter = list.iter().enumerate();
                iter.next();
                for (index, y) in iter {
                    left = match longer.as_bytes()[x-1] == shorter.as_bytes()[index] {
                        true => list[index-1],
                        false => [list[index-1], *y, left].iter().min().unwrap() + 1
                    };
                    temp.push(left);
                }
                if temp.iter().min().unwrap() > &distance {
                    return (*string_in, distance + 1)
                }
                list = temp
            }

            (*string_in, list.pop().unwrap() - 1)
        })
        .filter(|x| x.1 < distance)
        .collect();
    vec.par_sort_by(|a, b| a.1.cmp(&b.1));

    vec.par_iter().map(|x| x.0).collect()
}



#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    #[test]
    fn actual_dict() {
        let dictionary_string = include_str!("words.txt");
        let thing = crate::spellcheck(&dictionary_string, "restaraunt", 3);
        assert_eq!("restaurant", thing[0]);
    }

    #[bench]
    fn bench_actual_dict(bench: &mut Bencher) {
        let dictionary_string = include_str!("words.txt");
        bench.iter(||
            crate::spellcheck(&dictionary_string, "restaraunt", 3)
        )
    }

    #[test]
    fn fake_dict() {
        let string = "\
thin
thing
";
        assert_eq!(crate::spellcheck(string, "thinga", 3), vec!["thing", "thin"])
    }
}
