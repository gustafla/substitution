# Week 1 Report

## 2022-03-17, 3h

Today I started the project in a previous repository, in which I had toyed with
this concept before. I asked the teacher if this is a suitable project
for the course. Reading the instructions, initializing the repository and
writing documents took me maybe 3h of work (on and off).

I'm still a little unsure about how the program will work and what data
structures and algorithms it will utilize, but I wrote my best guesses in the
specification document for now.

Next week I'm planning to get [cargo-nextest](https://nexte.st) and
[tarpaulin](https://github.com/xd009642/tarpaulin) working in CLI and
Github Actions.

## 2022-03-18, 2h

I went ahead and registered the project in labtool and set up a couple of
tests, code coverage with tarpaulin and CI badges. Also added a CI workflow for
creating release-mode binaries for Windows, Mac and Linux, which will come in
handy for users who just want to try the program.
