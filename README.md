# find-simdoc

Find all pairs of similarity documents

## Example to run

```
cargo run --release -p find-simdoc --bin jaccard -- -i data/sample.txt -r 0.1 -w 5
cargo run --release -p find-simdoc --bin cosine -- -i data/sample.txt -r 0.1 -w 5
```
