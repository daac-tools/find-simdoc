use find_simdoc::tfidf::{Idf, Tf};
use find_simdoc::CosineSearcher;

fn main() {
    let documents = vec![
        "Welcome to Jimbocho, the town of books and curry!",
        "Welcome to Jimbocho, the city of books and curry!",
        "We welcome you to Jimbocho, the town of books and curry.",
        "Welcome to the town of books and curry, Jimbocho!",
    ];

    // Creates a searcher for word unigrams (with random seed value 42).
    let searcher = CosineSearcher::new(1, Some(' '), Some(42)).unwrap();
    // Creates a term frequency (TF) weighter.
    let tf = Tf::new();
    // Creates a inverse document frequency (IDF) weighter.
    let idf = Idf::new()
        .build(documents.iter().clone(), searcher.config())
        .unwrap();
    // Builds the database of binary sketches converted from input documents,
    let searcher = searcher
        // with the TF weighter and
        .tf(Some(tf))
        // the IDF weighter,
        .idf(Some(idf))
        // where binary sketches are in the Hamming space of 10*64 dimensions.
        .build_sketches_in_parallel(documents.iter(), 10)
        .unwrap();

    // Searches all similar pairs within radius 0.25.
    let results = searcher.search_similar_pairs(0.25);
    // A result consists of the left-side id, the right-side id, and their distance.
    assert_eq!(results, vec![(0, 1, 0.1671875), (0, 3, 0.246875)]);
}
