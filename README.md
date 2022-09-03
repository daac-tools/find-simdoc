# find-simdoc

Find all pairs of similarity documents

## Example to run

```
cargo run --release -p find-simdoc --bin jaccard -- -i data/sample.txt -r 0.1 -w 5 > jac.csv
cargo run --release -p find-simdoc --bin cosine -- -i data/sample.txt -r 0.1 -w 5 > cos.csv
```

```
cargo run --release -p find-simdoc --bin dump -- -i data/sample.txt -s jac.csv
cargo run --release -p find-simdoc --bin dump -- -i data/sample.txt -s cos.csv
```