//! ```
//! use spelling::spellcheck;
//! let dictionary_string = include_str!("words.txt"); // newline separated
//! spellcheck(dictionary_string.split("\n"), "restaraunt", 3);
//! ```
//! This uses the
//! [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
//! as the heuristic for distance.
#[cfg(feature = "use_rayon")]
use rayon::prelude::*;

#[cfg(feature = "use_rayon")]
pub fn spell_check_rayon<'a>(
    dictionary: impl IntoParallelIterator<Item = &'a str>,
    word: &str,
    max_distance: usize,
) -> Vec<&'a str> {
    let mut suggestions = dictionary
        .into_par_iter()
        .filter_map(|candidate| {
            let distance = levenshtein_distance_with_max(word, candidate, max_distance);
            distance.map(|distance| (distance, candidate))
        })
        .collect::<Vec<_>>();
    suggestions.par_sort_unstable_by_key(|&(distance, _)| distance);
    suggestions
        .into_iter()
        .map(|(_, suggestion)| suggestion)
        .collect()
}

pub fn spellcheck<'a>(
    dictionary: impl IntoIterator<Item = &'a str>,
    word: &str,
    max_distance: usize,
) -> Vec<&'a str> {
    let mut suggestions = dictionary
        .into_iter()
        .filter_map(|candidate| {
            let distance = levenshtein_distance_with_max(word, candidate, max_distance);
            distance.map(|distance| (distance, candidate))
        })
        .collect::<Vec<_>>();
    suggestions.sort_unstable_by_key(|&(distance, _)| distance);
    suggestions
        .into_iter()
        .map(|(_, suggestion)| suggestion)
        .collect()
}
/// Computes the Levenshtein distance, the minimum number of single-character edits (insertions, deletions, or substitutions)
/// required to change one string into the other.
pub fn levenshtein_distance<'a>(mut a: &'a str, mut b: &'a str) -> usize {
    // The Levenshtein distance is the minimum number of single-character edits (insertions, deletions, or substitutions)
    // required to change one string into the other.
    //
    // First, let us define the recursion:
    // lev(a, b) = cases(
    // |b|                         if |a| = 0,      (represents all insertions for a)
    // |a|                         if |b| = 0,      (represents all insertions for b)
    // lev(a[1..], b[1..])         if a[0] == b[0], (go to next char)
    // 1 + min(lev(a[1..], b[1..]),                 (first character must be replaced)
    //         lev(a[1..], b),                      (first character must be deleted)
    //         lev(a, b[1..]))                      (first character must be inserted)
    // )
    //
    // note that this always builds from the parts where the length of one of the sides is zero, where in that case
    // the return value is the length of the other side
    //
    // Now we should consider ordering.
    // We can ignore the base cases. Note what we refer to, we can therefore only use a single buffer to store the previous row.
    // (if you visualize the matrix, you can see we only need to keep track of the row and the value up and left)
    //
    let mut a_len = a.chars().count();
    let mut b_len = b.chars().count();

    if a_len > b_len {
        // Swap the strings to ensure that the shorter string is always 'a'
        (b, a) = (a, b);
        (b_len, a_len) = (a_len, b_len);
    }

    // a is always the shorter string
    let mut vector = Vec::with_capacity(a_len + 1);
    for i in 0..=a_len {
        vector.push(i);
    }

    let mut a_chars = a.chars();
    let mut b_chars = b.chars();
    for i in 0..b_len {
        let b_char = b_chars.next().unwrap();
        let mut up_left = vector[0];
        vector[0] = i + 1;
        for j in 1..=a_len {
            let a_char = a_chars.next().unwrap();
            if a_char == b_char {
                let sub_cost = up_left;
                up_left = vector[j];
                vector[j] = sub_cost;
            } else {
                let deletion_cost = vector[j] + 1;
                let insertion_cost = vector[j - 1] + 1;
                let sub_cost = up_left + 1;

                up_left = vector[j];
                vector[j] = [deletion_cost, insertion_cost, sub_cost]
                    .into_iter()
                    .min()
                    .unwrap();
            }
        }
        a_chars = a.chars();
    }

    return vector[a_len];
}

/// An optimized version of the Levenshtein distance that stops early if the distance exceeds max_distance
/// This is useful for spellchecking where we only care about suggestions within a certain distance, and can save time by not computing the full distance
/// When the distance exceeds max_distance, we will return an arbitrary number greater than max_distance.
pub fn levenshtein_distance_with_max<'a>(
    mut a: &'a str,
    mut b: &'a str,
    max_distance: usize,
) -> Option<usize> {
    // The Levenshtein distance is the minimum number of single-character edits (insertions, deletions, or substitutions)
    // required to change one string into the other.

    let mut a_len = a.chars().count();
    let mut b_len = b.chars().count();

    if a_len > b_len {
        // Swap the strings to ensure that the shorter string is always 'a'
        (b, a) = (a, b);
        (b_len, a_len) = (a_len, b_len);
    }

    if b_len - a_len > max_distance {
        return None; // distance is guaranteed to be greater than max_distance
    }
    let max_distance = max_distance as isize;
    // TODO: handle cases where 2 * max_distance - 1 > b_len to avoid "optimizing" when it won't help
    let k = max_distance * 2 - 1;
    let mut offset = 1 - max_distance;
    let mut vector = Vec::with_capacity(k as usize);
    let mut next_vector = Vec::with_capacity(k as usize);
    // iterate max_distance - 1 times to add padding
    for _ in 1..max_distance {
        // push large number so it is noticed if it is used
        vector.push(9);
    }
    // rest of the initial values
    for i in 0..max_distance {
        vector.push(i as usize);
    }

    for _ in 0..k {
        next_vector.push(9);
    }

    let mut a_chars = a.chars();
    // we will be doing a lot of indexing
    let b_chars = b.chars().collect::<Vec<_>>();
    // iterate over a
    for i in 0..a_len {
        let a_char = a_chars.next().unwrap();
        for j in 0..k {
            let char_index = j + offset;
            let j = j as usize;

            if char_index == -1 {
                next_vector[j] = i + 1;
                continue;
            } else if char_index < 0 || char_index >= b_len as isize {
                // we are outside the bounds of the string, do nothing
                continue;
            }

            let b_char = b_chars[char_index as usize];
            let cost = if a_char == b_char {
                let sub_cost = vector[j];
                sub_cost
            } else {
                let deletion_cost = if j as isize == k - 1 {
                    max_distance as usize
                } else {
                    vector[j + 1]
                } + 1;

                let insertion_cost = if j == 0 {
                    max_distance as usize
                } else {
                    next_vector[j - 1]
                } + 1;
                let sub_cost = vector[j] + 1;

                [deletion_cost, insertion_cost, sub_cost]
                    .into_iter()
                    .min()
                    .unwrap()
            };
            next_vector[j] = cost;
        }
        offset += 1;
        vector.copy_from_slice(&next_vector);

        if vector.iter().min().copied().unwrap() > max_distance as usize {
            return None;
        }
    }
    let result = if (k + offset) as usize <= b_len {
        vector.last().unwrap() + b_len - (k + offset) as usize
    } else {
        vector[b_len - offset as usize]
    };
    if result > max_distance as usize {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dist_max() {
        assert_eq!(
            levenshtein_distance_with_max("saturday", "sunday", 3),
            Some(3)
        );
        assert_eq!(levenshtein_distance_with_max("saturday", "sunday", 2), None);
        assert_eq!(levenshtein_distance_with_max("resta", "br", 3), None);
    }
    #[test]
    fn test_levenshtein_distance() {
        // kitten -> sitten -> sittin -> sitting
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        // flaw -> flawn -> lawn
        assert_eq!(levenshtein_distance("flaw", "lawn"), 2);
        // intention -> entention -> extention -> exention -> exection -> execution
        assert_eq!(levenshtein_distance("intention", "execution"), 5);
        // saturday -> sturday -> surday -> sunday
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);

        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("test", ""), 4);
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("rust", "rust"), 0);
        assert_eq!(levenshtein_distance("a", "a"), 0);
        assert_eq!(levenshtein_distance("a", "b"), 1);
        assert_eq!(levenshtein_distance("ac", "abc"), 1);
        assert_eq!(levenshtein_distance("abc", "ac"), 1);
        assert_eq!(levenshtein_distance("test", "testing"), 3);
        assert_eq!(levenshtein_distance("gumbo", "gambol"), 2);
        assert_eq!(levenshtein_distance("intention", "execution"), 5);
        assert_eq!(levenshtein_distance("book", "back"), 2);
        assert_eq!(levenshtein_distance("Test", "test"), 1);
        assert_eq!(levenshtein_distance("ORANGE", "orange"), 6);
        assert_eq!(levenshtein_distance("short", "a much longer string"), 16);
        assert_eq!(levenshtein_distance("a much longer string", "short"), 16);
        assert_eq!(levenshtein_distance("aaaaa", "bbbbb"), 5);
        assert_eq!(levenshtein_distance("ababab", "bababa"), 2);
        assert_eq!(levenshtein_distance("bbbaaa", "aaabbb"), 6);
        assert_eq!(levenshtein_distance("ab", "ba"), 2);
        assert_eq!(levenshtein_distance("algorithm", "logarithm"), 3);
        assert_eq!(levenshtein_distance("abcdef", "abcfed"), 2);
        assert_eq!(levenshtein_distance("crème brûlée", "creme brulee"), 3);
        assert_eq!(levenshtein_distance("你好世界", "你好"), 2);
        assert_eq!(levenshtein_distance("привет", "приветствие"), 5);
        assert_eq!(levenshtein_distance("a", "à"), 1);
        assert_eq!(levenshtein_distance("Schrödinger", "Schrodinger"), 1);
        assert_eq!(levenshtein_distance("İ", "I"), 1);
        assert_eq!(
            levenshtein_distance(
                "the quick brown fox jumps over the lazy dog",
                "the lazy brown dog jumps over the quick fox"
            ),
            14
        );
        assert_eq!(
            levenshtein_distance("this is a long sentence.", "this sentence is long."),
            15
        );
        assert_eq!(levenshtein_distance("1234567890", "0987654321"), 10);
    }
}