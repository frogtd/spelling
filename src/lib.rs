#![feature(test)]
//! ```
//! use spelling::spellcheck;
//! let dictionary_string = include_str!("words.txt"); // newline separated
//! spellcheck(dictionary_string, "restaraunt", 3);
//! ```
//! This uses the 
//! [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
//! as the heuristic for distance.


/// Takes a `dictionary_string` (newline separated), a word and a distance and
/// returns a vector of possible matches, with a limit of distance set up
/// `distance`. Sorts by distance. Uses rayon.
/// 
/// Notes:
/// 1. Use this whenever possible over the other `spellcheck` function.
/// 2. This only works for single length code bytes.
/// 3. This uses the Levenshtein distance.
/// ```
/// use spelling::spellcheck_rayon;
/// let dictionary_string = include_str!("words.txt"); // newline separated
/// spellcheck_rayon(dictionary_string, "restaraunt", 3);
/// ```
#[cfg(feature = "use_rayon")]
pub fn spellcheck_rayon<'a>(dictionary_string: &'a str, word: &str, distance: usize) -> Vec<&'a str> {
    use rayon::prelude::*;
    let mut vec: Vec<_> = dictionary_string
        .split('\n')
        .collect::<Vec<&str>>()
        .par_iter()
        .map(|string_in| {
            let (shorter, longer) = {
                if string_in.len() > word.len() {
                    (*string_in, word)
                } else {
                    (word, *string_in)
                }
            };

            // create list like [1, 1, 2, 3, 4, 5];
            let mut list: Vec<usize> = Vec::with_capacity(shorter.len() + 1);
            list.push(1);
            for index in 1..(shorter.len() + 1) {
                list.push(index)
            }

            for x in 2..(longer.len() + 1) {
                let mut left = x;
                let mut temp: Vec<usize> = Vec::with_capacity(shorter.len() + 1);
                temp.push(left);
                let mut iter = list.iter().enumerate();
                iter.next();
                for (index, y) in iter {
                    left = match longer.as_bytes()[x - 1] == shorter.as_bytes()[index - 1] {
                        true => list[index - 1],
                        false => [list[index - 1], *y, left].iter().min().unwrap() + 1,
                    };
                    temp.push(left);
                }

                // shortcircuit out 
                if temp.iter().min().unwrap() > &distance {
                    return (*string_in, distance + 1);
                }
                list = temp
            }

            (*string_in, list.pop().unwrap() - 1)
        })
        .filter(|x| x.1 < distance)
        .collect();
    
    // sort by distance and then return the words

    // TODO: this should really be a counting sort
    vec.par_sort_by(|a, b| a.1.cmp(&b.1));
    vec.par_iter().map(|x| x.0).collect()
}



/// Takes a `dictionary_string` (newline separated), a word and a distance and
/// returns a vector of possible matches, with a limit of distance set up
/// `distance`. Sorts by distance. This doesn't use rayon.
/// 
/// Notes:
/// 1. This only works for single length code bytes.
/// 2. This uses the Levenshtein distance.
/// ```
/// use spelling::spellcheck;
/// let dictionary_string = include_str!("words.txt"); // newline separated
/// spellcheck(dictionary_string, "restaraunt", 3);
/// ```
/// How it works:
///
/// It loops through each word in the dictionary and then you have a word from
/// the dictionary and a word to match against.
///
/// Lets say those words were `rumor` and `rotten`. You then create a vector
/// that is the length of the shorter word + 1, counting up (but the first value
/// is one).
/// ```
/// let mut list = vec![1, 1, 2, 3, 4, 5];
/// ```
/// You also keep track a left variable, which starts at the index of the
/// longest string you're at and you create a temporary row.
/// ```
/// # let longer = "rotten";
/// for x in 2..longer.len() {
///     let mut left = x;
///     let mut temp = Vec::new();
///     # temp.push(10 as usize)
/// }
/// ```
/// For every item in the list, except the first, check if letter you're on 
/// is the same in both words. If it is left will become list at the current
/// index you're on minus one. If it isn't left will become the minimum of 
/// the current index on `list`, the current index minus on `list`, and left. 
/// Then add `left` to `temp`.
/// 
/// After this set `list` to `temp` and repeat.
/// 
/// The distance will be in the last index of list at the end, so return that
/// to filter.
///
/// ```
/// # let longer = "rotten";
/// # let shorter = "rumor";
/// # let mut list: Vec<usize> = Vec::with_capacity(shorter.len() + 1);
/// # list.push(1);
/// # for index in 1..(shorter.len() + 1) {
/// #     list.push(index) 
/// # }
/// for x in 2..longer.len() {
///     let mut left = x;
///     println!("{}", x);
///     let mut temp = Vec::new();
///     temp.push(left);
///     let mut iter = list.iter().enumerate();
///     iter.next(); // skip first item in list
///     for (index, y) in iter {
///         println!("{}", index);
///         left = match longer.as_bytes()[x - 1] == shorter.as_bytes()[index - 1] {
///             true => list[index - 1],
///             false => [list[index - 1], *y, left].iter().min().unwrap() + 1,
///         };
///         temp.push(left)
///     }
///     list = temp;
/// }
/// ```
///
pub fn spellcheck<'a>(dictionary_string: &'a str, word: &str, distance: usize) -> Vec<&'a str> {
    let mut out = Vec::new();
    'loop1: for string_in in dictionary_string.split("\n") {
        let (shorter, longer) = {
            if string_in.len() > word.len() {
                (string_in, word)
            } else {
                (word, string_in)
            }
        };

        // create list like [1, 1, 2, 3, 4, 5];
        let mut list: Vec<usize> = Vec::with_capacity(shorter.len() + 1);
        list.push(1);
        for index in 1..(shorter.len() + 1) {
            list.push(index)
        }

        for x in 2..(longer.len() + 1) {
            let mut left = x;
            let mut temp: Vec<usize> = Vec::with_capacity(shorter.len() + 1);
            temp.push(left);
            let mut iter = list.iter().enumerate();
            iter.next();
            for (index, y) in iter {
                left = match longer.as_bytes()[x - 1] == shorter.as_bytes()[index - 1] {
                    true => list[index - 1],
                    false => [list[index - 1], *y, left].iter().min().unwrap() + 1,
                };
                temp.push(left);
            }

            // shortcircuit out 
            if temp.iter().min().unwrap() > &distance {
                continue 'loop1;
            }
            list = temp
        }
        let out_distance = list.pop().unwrap() - 1;
        if out_distance < distance {
            out.push((string_in, out_distance))
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out.iter().map(|x| x.0).collect()
} 

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;
    #[test]
    #[cfg(feature = "use_rayon")]
    fn actual_dict() {
        let dictionary_string = include_str!("words.txt");
        let thing = crate::spellcheck_rayon(&dictionary_string, "restaraunt", 3);
        assert_eq!("restaurant", thing[0]);
    }

    #[bench]
    #[cfg(feature = "use_rayon")]
    fn bench_actual_dict(bench: &mut Bencher) {
        let dictionary_string = include_str!("words.txt");
        bench.iter(|| crate::spellcheck_rayon(&dictionary_string, "restaraunt", 3))
    }

    #[test]
    #[cfg(feature = "use_rayon")]
    fn fake_dict() {
        let string = "\
thin
thing
";
        assert_eq!(
            crate::spellcheck_rayon(string, "thinga", 3),
            vec!["thing", "thin"]
        )
    }

    #[test]
    fn no_rayon_actual_dict() {
        let dictionary_string = include_str!("words.txt");
        let thing = crate::spellcheck(&dictionary_string, "restaraunt", 3);
        assert_eq!("restaurant", thing[0]);
    }

    #[bench]
    fn no_rayon_bench_actual_dict(bench: &mut Bencher) {
        let dictionary_string = include_str!("words.txt");
        bench.iter(|| crate::spellcheck(&dictionary_string, "restaraunt", 3))
    }

    #[test]
    fn no_rayon_fake_dict() {
        let string = "\
thin
thing
";
        assert_eq!(
            crate::spellcheck(string, "thinga", 3),
            vec!["thing", "thin"]
        )
    }
}
