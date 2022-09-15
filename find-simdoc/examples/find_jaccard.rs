use find_simdoc::JaccardSearcher;

fn main() {
    let documents = vec![
        "Welcome to Jimbocho, the town of books and curry!",
        "Welcome to Jimbocho, the city of books and curry!",
        "We welcome you to Jimbocho, the town of books and curry.",
        "Welcome to the town of books and curry, Jimbocho!",
    ];

    // Creates a searcher for character trigrams (with random seed value 42).
    let searcher = JaccardSearcher::new(3, None, Some(42))
        .unwrap()
        // Builds the database of binary sketches converted from input documents,
        // where binary sketches are in the Hamming space of 20*64 dimensions.
        .build_sketches_in_parallel(documents.iter(), 20)
        .unwrap();

    // Searches all similar pairs within radius 0.25.
    let results = searcher.search_similar_pairs(0.25);
    assert_eq!(results, vec![(0, 1, 0.1875), (0, 3, 0.2296875)]);
}
