#!/usr/bin/env python3
'''
Download NLTK corpora and create a text file of sentences with lowercase letters and no duplicate lines.
'''

import sys
import nltk
from nltk.corpus import reuters
from argparse import ArgumentParser


def download_reuters():
    nltk.download('reuters')
    nltk.download('punkt')
    return reuters.sents()


def main():
    parser = ArgumentParser()
    parser.add_argument('name')
    args = parser.parse_args()

    if args.name == 'reuters':
        sents = download_reuters()
    else:
        print(f'unsupported corpus name: {args.name}', file=sys.stderr)
        return

    with open(f'{args.name}.txt', 'wt') as fout:
        sents = [' '.join(sent).lower() for sent in sents]
        for sent in set(sents):
            fout.write(sent)
            fout.write('\n')


if __name__ == "__main__":
    main()
