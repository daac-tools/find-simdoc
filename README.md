# Finding similar documents

This software provides fast all-pair similarity searches in documents.

## Problem definition

- Input
  - List of documents $D = (d_1, d_2, \dots, d_n)$
  - Distance function $\delta: D \times D \rightarrow [0,1]$
  - Radius threshold $r \in [0,1]$
- Output
  - All pairs of similar document ids $R = \\{ (i,j): i < j, \delta(d_i, d_j) \leq r \\}$

## Features

- **Easy to use:** This software supports all essential steps of document similarity search, from feature extraction to output of similar pairs. Therefore, you can immediately try the fast all-pair similarity search using your document files.
- **Flexible tokenization:** You can specify any delimiter when splitting words in tokenization. This can be useful in languages where multiple definitions of words exist, such as Japanese or Chinese.
- **Time- and memory-efficient:** The time and space complexities are *linear* over the numbers of input documents and output results, based on the idea of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html).
- **Pure Rust:** This software is implemented in Rust, achieving safe and fast performance.

## Running example

Here, we describe the basic usage of this software through a running example.

First of all, install `rustc` and `cargo` following the [official instructions](https://www.rust-lang.org/tools/install) since this software is implemented in Rust.

### 1. Data preparation

You have to prepare a text file containing documents line by line.

To produce an example file used throughout this description, you can use `scripts/load_nltk_dataset.py` that downloads the Reuters Corpus provided by NLTK.
Run the following command.

```
$ ./scripts/load_nltk_dataset.py reuters
```

`reuters.txt` will be output.

```
$ head reuters.txt
hre properties & lt ; hre > 1st qtr jan 31 net shr 38 cts vs 47 cts net 2 , 253 , 664 vs 2 , 806 , 820 gross income 5 , 173 , 318 vs 5 , 873 , 904 note : net includes gains on sale of real estate of 126 , 117 dlrs vs 29 , 812 dlrs .
the firm , however , is supplying temporary financing , and sources close to the transaction disputed the claim that the firm will not end up paying for its equity position . 
conoco , which has completed geological prospecting for the tunisian government , has transferred one third of its option rights in the region to ina , it said .
" willis faber ' s stake in morgan grenfell has been a very successful investment ," it said .
china reports 700 mln dlr two - month trade deficit china ' s trade deficit totalled 700 mln dlrs in the first two months of this year , according to figures released by the state statistics bureau .
the treasury said baker and stoltenberg " are consulting with their g - 7 colleagues and are confident that this will enable them to foster exchange rate stability around current levels ."
u . s . tariffs are due to take effect on april 17 .
some dealers said there were growing signs the united states wanted the dollar to fall further .
since last august smart has been leading talks to open up japan to purchases of more u . s .- made automotive parts .
the resulting association will operate under the name of charter and will be based in bristol .
```

Fully-duplicate documents in `reuters.txt` are removed because they are noisy in evaluation of similarity searches.
To do this, the output lines are shuffled, and your file will not be the identical to the example.

### 2. Finding all pairs of similar documents

The workspace `find-simdoc` provides CLI tools for fast all-pair similarity searches in documents.
The approach consists of three steps:

1. Extract features from documents
   - Set representation of character ngrams
   - Set representation of word ngrams
2. Convert the features into binary sketches through locality sensitive hashing (LSH)
   - [1-bit minwise hashing](https://arxiv.org/abs/0910.3349) for the Jaccard similarity
   - [Simplified simhash](https://dl.acm.org/doi/10.1145/1242572.1242592) for the Cosine similarity
3. Search for similar sketches in the Hamming space using a modified variant of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html)

Note that the current version supports only set representations in Step 1.
Supporting weighting approaches such as TF-IDF is the future work.

#### 2.1 Jaccard space

The executable `jaccard` provides a similarity search in the [Jaccard space](https://en.wikipedia.org/wiki/Jaccard_index).
You can check the arguments with the following command.

```
$ cargo run --release -p find-simdoc --bin jaccard -- --help
```

Run the following command if you want to search for `reuters.txt` with

- search radius `0.1`,
- tokens of character `5`-grams, and
- `8*64=512` dimensions in the Hamming space.

```
$ cargo run --release -p find-simdoc --bin jaccard -- -i reuters.txt -r 0.1 -w 5 -c 8 > result-jaccard.csv
```

Argument `-c` indicates the number of dimensions in the Hamming space,
a trade-off parameter between approximation accuracy and search speed.
The larger this value, the higher the accuracy, but the longer the search takes.
[This section](#4-testing-the-accuracy-of-1-bit-minwise-hashing) describes how to examine the approximation accuracy for the number of dimensions.

Pairs of similar documents (indicated by zero-origin line numbers) and their distances are reported.

```
$ head result-jaccard.csv
i,j,dist
31,1357,0.05859375
173,49999,0.05078125
286,19892,0.07421875
308,51423,0.015625
371,47578,0.03125
403,5338,0.0859375
792,48327,0.078125
839,50322,0.09765625
1250,43620,0.08984375
```

#### 2.2 Cosine space

The executable `cosine` provides a similarity search in the [Cosine space](https://en.wikipedia.org/wiki/Cosine_similarity).
You can check the arguments with the following command.

```
$ cargo run --release -p find-simdoc --bin cosine -- --help
```

Run the following command if you want to search for `reuters.txt` with

- search radius `0.15`,
- tokens of word `3`-grams
- word delimiter `" "` (i.e., a space), and
- `4*64=256` dimensions in the Hamming space.

```
$ cargo run --release -p find-simdoc --bin cosine -- -i reuters.txt -r 0.15 -d " " -w 3 -c 4 > result-cosine.csv
```

Pairs of similar documents (indicated by zero-origin line numbers) and their distances are reported.

```
$ head result-cosine.csv
i,j,dist
31,1357,0.11328125
173,49999,0.10546875
308,51423,0.12890625
371,47578,0.06640625
448,27050,0.08203125
457,18390,0.140625
792,48327,0.1484375
806,43538,0.140625
844,30595,0.11328125
```

### 3. Printing similar documents

The executable `dump` prints similar documents from an output CSV file.

If you want to print similar documents in `reuters.txt` with the result `result-jaccard.csv`,
run the following command.

```
$ cargo run --release -p find-simdoc --bin dump -- -i reuters.txt -s result-jaccard.csv
[i=31,j=1357,dist=0.05859375]
the january fall came after a strong 6 . 4 pct rise from november ' s rate of 1 . 774 mln units and brought completions to 6 . 7 pct above the january , 1986 , level of 1 . 765 mln units .
the january fall came after a strong 6 . 4 pct rise from november ' s rate of 1 . 774 mln units and brought completions to 6 . 7 pct above the january , 1986 level of 1 . 765 mln units .
[i=173,j=49999,dist=0.05078125]
rio de janeiro dockers strike over wages rio de janeiro ' s 3 , 500 dockworkers went on strike for an indefinite period today to demand wage increases , a spokesman for the dockers said .
rio de janeiro dockers strike rio de janeiro ' s 3 , 500 dockworkers went on strike for an indefinite period today to demand wage increases , a spokesman for the dockers said .
[i=286,j=19892,dist=0.07421875]
u . s . first time jobless claims rose in week new applications for unemployment insurance benefits rose to a seasonally adjusted 355 , 000 in the week ended march 21 from 341 , 000 in the prior week , the labor department said .
u . s . first time jobless claims rose in week new applications for unemployment insurance benefits rose to a seasonally adjusted 341 , 000 in the week ended march 14 from 340 , 000 in the prior week , the labor department said .
[i=308,j=51423,dist=0.015625]
nigeria changes auction rules to defend naira nigeria ' s central bank has changed the rules governing its foreign exchange auctions in what analysts see as a means of defending the naira currency , which has depreciated steadily .
nigeria changes auction rules to defend the naira nigeria ' s central bank has changed the rules governing its foreign exchange auctions in what analysts see as a means of defending the naira currency , which has depreciated steadily .
[i=371,j=47578,dist=0.03125]
" the administration should communicate to the european community the message that the united states will view the establishment of such a tax as inconsistent with the european community ' s obligations under the general agreement on tariffs and trade that will result in the adoption of strong and immediate countermeasures ," the resolution stated .
" the administration should communciate to the european community the message that the united states will view the establishment of such a tax as inconsistent with the european community ' s obligations under the general agreement on tariffs and trade that will result in the adoption of strong and immediate countermeasures ," the resolution stated .
[i=403,j=5338,dist=0.0859375]
he forecast the chancellor ' s budget tax cuts would increase consumer expediture on imported goods .
he forecast the chancellor ' s budget tax cuts would increase consumer expenditure on imported goods .
...
```

### 4. Testing the accuracy of 1-bit minwise hashing

LSH is an approximate solution, and you may want to know the accuracy.
The executable `minhash_acc` allows you to examine
- the *mean absolute error* that is the averaged gap between the normalized Hamming distance and the actual Jaccard distance,
- the precisions for search radii {0.1, 0.2, 0.5},
- the recalls for search radii {0.1, 0.2, 0.5}, and
- the F1-scores for search radii {0.1, 0.2, 0.5}.

To use this executable, we recommend extracting a small subset from your dataset
because it exactly computes distances for all possible pairs and maintains a large amount of data in main memory.

```
$ head -5000 reuters.txt > reuters.5k.txt
```

You can test the number of Hamming dimensions from 64 to 6400
(i.e., the number of chunks from 1 to 100 indicated with `-c`)
with the following command.
The arguments for feature extraction are the same as those of `jaccard`.

```
$ cargo run --release -p find-simdoc --bin minhash_acc -- -i reuters.5k.txt -w 5 > acc.csv
```

The statistics will be reported as follows.
It can be seen that the accuracy improves as the number of dimensions increases.

```
$ cat acc.csv
num_chunks,dimensions,mean_absolute_error,precision_0.1,recall_0.1,f1_0.1,precision_0.2,recall_0.2,f1_0.2,precision_0.5,recall_0.5,f1_0.5
1,64,0.09996782775866102,0.5,1,0.6666666666666666,0.8,0.5714285714285714,0.6666666666666666,0.02139917695473251,0.7647058823529411,0.04163330664531626
2,128,0.07069611162104494,1,1,1,0.8333333333333334,0.7142857142857143,0.7692307692307692,0.47368421052631576,0.7941176470588235,0.5934065934065933
3,192,0.05776715360376704,1,1,1,0.8333333333333334,0.7142857142857143,0.7692307692307692,0.6304347826086957,0.8529411764705882,0.725
4,256,0.049919892543492474,1,1,1,1,0.7142857142857143,0.8333333333333333,0.8181818181818182,0.7941176470588235,0.8059701492537314
...
97,6208,0.0101065034709229,1,1,1,1,1,1,1,0.9705882352941176,0.9850746268656716
98,6272,0.010056169962171922,1,1,1,1,1,1,1,0.9705882352941176,0.9850746268656716
99,6336,0.01000512597491915,1,1,1,1,1,1,1,0.9705882352941176,0.9850746268656716
100,6400,0.00995601577231552,1,1,1,1,1,1,1,0.9705882352941176,0.9850746268656716
```

## Disclaimer

This software is developed by LegalForce, Inc.,
but not an officially supported LegalForce product.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## TODO

- Add threading
- Add TF-IDF weighting
- Derive the complexity
