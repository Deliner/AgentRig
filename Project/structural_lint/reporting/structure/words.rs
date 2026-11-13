const STOPWORDS: &[&str] = &[
    "about", "after", "all", "an", "and", "any", "are", "as", "at", "be", "before", "both", "by",
    "can", "does", "each", "every", "for", "from", "has", "have", "how", "if", "in", "into", "is",
    "it", "its", "may", "must", "no", "not", "of", "on", "one", "only", "or", "other", "own",
    "per", "so", "such", "than", "that", "the", "their", "them", "then", "there", "these", "this",
    "those", "through", "to", "under", "up", "via", "was", "were", "what", "when", "where",
    "which", "while", "who", "why", "will", "with", "within", "without",
];
const PLURALS: [(&str, &str); 6] = [
    ("ies", "y"),
    ("ied", "y"),
    ("sses", "ss"),
    ("ches", "ch"),
    ("shes", "sh"),
    ("xes", "x"),
];
const DERIVATIONS: [(&str, &str); 14] = [
    ("ification", ""),
    ("ization", "ize"),
    ("isation", "ize"),
    ("ational", ""),
    ("ibility", "ible"),
    ("ability", "able"),
    ("ation", ""),
    ("ator", ""),
    ("ment", ""),
    ("ness", ""),
    ("ity", ""),
    ("ify", ""),
    ("ion", ""),
    ("ate", ""),
];
const SUFFIXES: [&str; 4] = ["ing", "ed", "er", "e"];

pub fn split(raw: &str) -> Vec<String> {
    let mut words = Vec::new();
    for part in raw.split(|c: char| !c.is_ascii_alphanumeric()) {
        let characters: Vec<char> = part.chars().collect();
        let mut current = String::new();
        for (index, &character) in characters.iter().enumerate() {
            let previous = index.checked_sub(1).and_then(|at| characters.get(at));
            let next = characters.get(index + 1);
            let lower_before = previous.is_some_and(char::is_ascii_lowercase);
            let acronym_end = previous.is_some_and(char::is_ascii_uppercase)
                && next.is_some_and(char::is_ascii_lowercase);
            let after_word = lower_before || acronym_end;
            let boundary = character.is_ascii_uppercase() && after_word;
            let digit = character.is_ascii_digit();
            let ends_word = boundary || digit;
            if ends_word {
                words.push(std::mem::take(&mut current));
            }
            let letter = !digit;
            if letter {
                current.push(character.to_ascii_lowercase());
            }
        }
        words.push(current);
    }
    words.retain(|word| !word.is_empty());
    words
}

pub fn stem(word: &str) -> Option<String> {
    let insignificant = word.len() < 2 || STOPWORDS.contains(&word);
    if insignificant {
        return None;
    }
    let mut stem = singular(word);
    let derived = DERIVATIONS
        .iter()
        .find(|(suffix, _)| stem.len() >= suffix.len() + 3 && stem.ends_with(suffix));
    if let Some((suffix, replacement)) = derived {
        stem.truncate(stem.len().saturating_sub(suffix.len()));
        stem.push_str(replacement);
    }
    for suffix in SUFFIXES {
        let removable = stem.len() >= suffix.len() + 3 && stem.ends_with(suffix);
        if removable {
            stem.truncate(stem.len().saturating_sub(suffix.len()));
        }
    }
    let mut tail = stem.bytes().rev();
    let (last, before) = (tail.next(), tail.next());
    let doubled =
        stem.len() > 3 && last == before && last.is_some_and(|b| !b"aeioulsz".contains(&b));
    if doubled {
        stem.pop();
    }
    Some(stem)
}

fn singular(word: &str) -> String {
    let plural = PLURALS
        .iter()
        .find(|(suffix, _)| word.len() > suffix.len() + 1 && word.ends_with(suffix));
    let singular_s = word.len() > 3
        && word.ends_with('s')
        && !["ss", "us", "is"].iter().any(|end| word.ends_with(end));
    let mut base = word.to_owned();
    if let Some((suffix, replacement)) = plural {
        base.truncate(base.len().saturating_sub(suffix.len()));
        base.push_str(replacement);
    } else if singular_s {
        base.pop();
    }
    base
}

pub fn stems(name: &str) -> Vec<String> {
    split(name).iter().filter_map(|word| stem(word)).collect()
}
