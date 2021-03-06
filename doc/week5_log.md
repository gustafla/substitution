# Week 5 Report

## 2022-04-10, 5h

I spent a surprisingly long time adding a few tests to my trie and refactoring
it's API. I also implemented dictionary file loading into `decrypt()`.

## 2022-04-11, 1h

I did some finishing touches to yesterday's work and wrote this report.

## 2022-04-12, 5h

I attended a workshop and implemented some decryption key logic. I don't yet
have the big picture of how everything will fit together but I'm trying to do
what I can, one thing at a time.

## 2022-04-13, 3h

I played around with the decoding code but did not get the pieces together yet.

## 2022-04-14, 5h

I tried a recursive approach. I'm thinking of ditching the frequency order
for a simpler brute force.

## 2022-04-15, 8h

I got the recursive approach to work but also reintroduced the frequency order
as it was clearly much faster. I'm now starting to see results but the
deciphers can still take a long time. I also cleaned up the codebase a little,
but I feel it's still somewhat messy.

Next I think I can optimize by going through the input words in length order.
I need to try different strategies.

I also need to write more documentation.

## 2022-04-16, 5h

I wrote docs. I believe the project is currently in good shape to be reviewed.

... And I couldn't resist partycoding at [Instanssi](https://instanssi.org) and
I managed to improve the smarts of the decipher by a whole lot.

Processing input words in order of number of unique characters nearest to
7 (so that key permutations are 26^7 = 8 billion) turns out to be a pretty
good strategy.

I also added functionality to skip 10% of input words because not everything is
in the dictionary.

Next I want to benchmark and write docs about benchmarking. And also do my peer
review.
