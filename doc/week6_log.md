# Week 6 Report

## 2022-04-28, 1h

I wrote a peer review for other student's project. I added some comments which
describe the decryption related function logic and fixed a flaky test.

There are several sort of "tickets" that I could work on next.

- Improving the decryption further. Right now most long inputs take a long time
  to decrypt. I suspect that is because the "word skip" introduces even more
  complexity (frequently going deep up and down the stack) and I wonder if the
  same effect could be implemented better.
  
- Adding support for user-defined alphabets (and frequency orders). This
  shouldn't require too much work as the same trie and logic would mostly work.
  Biggest changes are needed around the input and output. Checking that
  the cardinality of the input alphabet doesn't exceed the cardinality of the
  given alphabet would be a smart pre-check, for example.
  
- Writing better tests. The current tests are not covering all of the
  word skipping functionality for some reason. This might be a tarpaulin bug.
  
- Get a Github action for release build binaries to work again, preferably
  a custom one as the previously used one didn't keep up to date with Rust.