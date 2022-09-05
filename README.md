# Finding similar documents

This software provides fast all-pair similarity searches in a document file.

## Example of finding similar sentences

This software is implemented in Rust.
First of all, install `rustc` and `cargo` following the [official instructions](https://www.rust-lang.org/tools/install).

### 1. Data preparation

You have to prepare a document file containing search sentences line by line.

From the Reuters Corpus provided by NLTK, you can produce the document file used throughout this example, with the following command.

```
$ ./scripts/load_nltk_sents.py reuters
```

`reuters.txt` will be output.
Note that, since lines are shuffled to deduplicate sentences, your file will not be the identical to the following example.

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

### 2. Finding all pairs of similar sentences

The workspace `find-simdoc` provides CLI tools for fast all-pair similarity searches in a document file.
The approach consists of three steps:

1. Extract features from sentences
   - Set representation of character q-grams
   - Set representation of word q-grams
2. Convert the features into binary sketches through locality sensitive hashing (LSH)
   - [1-bit minwise hashing](https://arxiv.org/abs/0910.3349) for the Jaccard similarity
   - [Simplified simhash](https://dl.acm.org/doi/10.1145/1242572.1242592) for the Cosine similarity
3. Search for similar sketches using a modified variant of the [sketch sorting approach](https://proceedings.mlr.press/v13/tabei10a.html)

Note that the current version supports only set representations in Step 1.
Supporting weighting approaches such as TF-IDF is the future work.

#### In the Jaccard space

The executable `jaccard` provides a similarity search in the [Jaccard space](https://en.wikipedia.org/wiki/Jaccard_index).
You can check the arguments with the following command.

```
$ cargo run --release -p find-simdoc --bin jaccard -- --help
```

If you want to find similar sentences in `reuters.txt` within search radius `0.1` for tokens of
character `5`-grams, run the following command.

```
$ cargo run --release -p find-simdoc --bin jaccard -- -i reuters.txt -r 0.1 -w 5 > result-jaccard.csv
```

Pairs of similar sentences (indicated by line numbers) and their distances are reported.

```
$ head result-jaccard.csv
i,j,dist
31,1357,0.03125
103,50206,0.05859375
308,51423,0.00390625
371,47578,0.05078125
403,5338,0.0703125
839,20916,0.08984375
839,43949,0.09375
839,50322,0.09765625
1250,43620,0.09765625
```

#### In the Cosine space

The executable `cosine` provides a similarity search in the [Cosine space](https://en.wikipedia.org/wiki/Cosine_similarity).
You can check the arguments with the following command.

```
$ cargo run --release -p find-simdoc --bin cosine -- --help
```

If you want to find similar sentences in `reuters.txt` within search radius `0.15` for tokens of
word `3`-grams (separated by a space), run the following command.

```
$ cargo run --release -p find-simdoc --bin cosine -- -i reuters.txt -r 0.15 -d " " -w 3 > result-cosine.csv
```

Pairs of similar sentences (indicated by line numbers) and their distances are reported.

```
$ head result-cosine.csv
i,j,dist
31,1357,0.1015625
93,38484,0.12890625
173,49999,0.103515625
308,51423,0.109375
371,47578,0.091796875
1243,8907,0.14453125
1250,42018,0.130859375
1486,39803,0.14453125
1585,6615,0.13671875
```

### 3. Printing similar sentences

The executable `dump` prints similar sentences from an output CSV file.

If you want to print similar sentences in `reuters.txt` with the result `result-jaccard.csv`,
run the following command.

```
$ cargo run --release -p find-simdoc --bin dump -- -i reuters.txt -s result-jaccard.csv
[i=31,j=1357,dist=0.03125]
the january fall came after a strong 6 . 4 pct rise from november ' s rate of 1 . 774 mln units and brought completions to 6 . 7 pct above the january , 1986 , level of 1 . 765 mln units .
the january fall came after a strong 6 . 4 pct rise from november ' s rate of 1 . 774 mln units and brought completions to 6 . 7 pct above the january , 1986 level of 1 . 765 mln units .
[i=103,j=50206,dist=0.05859375]
the terms of the transaction were not disclosed .
terms of the transaction were not disclosed .
[i=308,j=51423,dist=0.00390625]
nigeria changes auction rules to defend naira nigeria ' s central bank has changed the rules governing its foreign exchange auctions in what analysts see as a means of defending the naira currency , which has depreciated steadily .
nigeria changes auction rules to defend the naira nigeria ' s central bank has changed the rules governing its foreign exchange auctions in what analysts see as a means of defending the naira currency , which has depreciated steadily .
[i=371,j=47578,dist=0.05078125]
" the administration should communicate to the european community the message that the united states will view the establishment of such a tax as inconsistent with the european community ' s obligations under the general agreement on tariffs and trade that will result in the adoption of strong and immediate countermeasures ," the resolution stated .
" the administration should communciate to the european community the message that the united states will view the establishment of such a tax as inconsistent with the european community ' s obligations under the general agreement on tariffs and trade that will result in the adoption of strong and immediate countermeasures ," the resolution stated .
[i=403,j=5338,dist=0.0703125]
he forecast the chancellor ' s budget tax cuts would increase consumer expediture on imported goods .
he forecast the chancellor ' s budget tax cuts would increase consumer expenditure on imported goods .
...
```

### 4. Testing the accuracy of 1-bit minwise hashing

LSH is an approximate solution, and you may want to know the accuracy.

The executable `minhash_mae` allows you to examine the *mean absolute error (MAE)*,
the averaged gap between the normalized Hamming distance with the minwise hashing
and the actual Jaccard distance.

To use this executable, we recommend extracting a small subset from your dataset
because it computes distances for all possible pairs.

```
$ head -5000 reuters.txt > reuters.5k.txt
```

```
$ cargo run --release -p find-simdoc --bin minhash_mae -- -i reuters.5k.txt -w 5 > mae.csv
```

```
$ cat mae.csv
num_chunks,dimensions,mean_absolute_error
1,64,0.09974628492462442
2,128,0.07050781338677266
3,192,0.05761297836012548
4,256,0.049865352075419325
...
97,6208,0.010101573974127143
98,6272,0.010049751534166197
99,6336,0.009999685515430031
100,6400,0.009950974569090776
```

## TODO

- Add threading for `chunked_join`
- Add TF-IDF weighting
- Derive the complexity
