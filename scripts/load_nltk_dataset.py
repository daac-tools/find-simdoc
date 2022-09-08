#!/usr/bin/env python3
'''
Download NLTK corpora and create a text file of sentences with lowercase letters and no duplicate lines.
'''

import sys
import nltk
from argparse import ArgumentParser


def download_reuters():
    from nltk.corpus import reuters
    nltk.download('reuters')
    return reuters.sents()


def download_gutenberg():
    from nltk.corpus import gutenberg
    nltk.download('gutenberg')
    return gutenberg.sents()


def download_webtext():
    from nltk.corpus import webtext
    nltk.download('webtext')
    return webtext.sents()


def download_brown():
    from nltk.corpus import brown
    nltk.download('brown')
    return brown.sents()


def download_inaugural():
    from nltk.corpus import inaugural
    nltk.download('inaugural')
    return inaugural.sents()


def main():
    parser = ArgumentParser()
    parser.add_argument('name')
    args = parser.parse_args()

    nltk.download('punkt')

    if args.name == 'reuters':
        sents = download_reuters()
    elif args.name == 'gutenberg':
        sents = download_gutenberg()
    elif args.name == 'webtext':
        sents = download_webtext()
    elif args.name == 'brown':
        sents = download_brown()
    elif args.name == 'inaugural':
        sents = download_inaugural()
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
