# find-simdoc

This is software for quickly finding all pairs of similar documents.

## Example of finding similar documents

This software is implemented in Rust.
First of all, install `rustc` and `cargo` following the [official instructions](https://www.rust-lang.org/tools/install).

### Data preparation

You can prepare the document file used in this example using `scripts/load_nltk_sents.py`.
From the Reuters Corpus provided by NLTK, you can produce line-separated sentences
with the following command.

```
$ ./scripts/load_nltk_sents.py reuters
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

### Finding all pairs of similar sentences

#### In the Jaccard space

```
$ cargo run --release -p find-simdoc --bin jaccard -- -i reuters.txt -r 0.1 -w 5 > result-jaccard.csv
```

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

```
$ cargo run --release -p find-simdoc --bin cosine -- -i reuters.txt -r 0.15 -d " " -w 3 > result-cosine.csv
```

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

### Printing similar sentences

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

## TODO

- Add threading for chunked_join
- Add TF-IDF weights
- Add a tool to evaluate accuracy of minhash
