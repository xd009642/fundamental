# fundamental

Fund the people fundamental to your success. Fundamental lets you pick a
project published to crates.io, and then it will go through it's dependencies
to a maximum depth and find all the sponsor links, project and user!

This project was inspired by a thread I read about open source funding and some
suggestions there. It's just a quick PoC I've done to prove the concept.
Currently, I don't plan on doing much more on it or publishing it to crates.io
but that could change if it's actually a good idea :eyes:.

You can also specify whether to include dev-dependencies, see how many
contributions each person is responsible for, how many crates in the
dependency tree they've contributed to and how many sponsors they have.
You can also specify on whether to sort by contributions (default descending),
or number of sponsors (default ascending). My hope for this is you can find
the people most directly responsible for the projects you love or the people
who seem underappreciated.

This does call against the github API so you'll need an API token
in the env var `GITHUB_API_TOKEN`.

Example output:

```
$ fundamental -i cargo-tarpaulin --max-depth 1

You can sponsor these projects directly!
=========================================
https://github.com/xd009642/tarpaulin links: ["https://github.com/xd009642", "https://patreon.com/xd009642"]
https://github.com/dtolnay/syn links: ["https://github.com/dtolnay"]
https://github.com/clap-rs/clap links: ["https://opencollective.com/clap"]
https://github.com/Kimundi/rustc-version-rs links: ["https://github.com/djc", "https://patreon.com/dochtman"]
https://github.com/sfackler/rust-fallible-iterator links: ["https://github.com/sfackler"]
https://github.com/dtolnay/quote links: ["https://github.com/dtolnay"]

You can sponsor these users for their work!
============================================
http://github.com/dtolnay (6903 contributions) (8 crates) (158 sponsors)
http://github.com/kbknapp (1745 contributions) (1 crates) (3 sponsors)
http://github.com/hawkw (1302 contributions) (2 crates) (19 sponsors)
http://github.com/xd009642 (1277 contributions) (4 crates) (14 sponsors)
http://github.com/JohnTitor (451 contributions) (2 crates) (15 sponsors)
http://github.com/sfackler (131 contributions) (5 crates) (13 sponsors)
http://github.com/seanmonstar (94 contributions) (1 crates) (40 sponsors)
http://github.com/joshtriplett (70 contributions) (2 crates) (20 sponsors)
http://github.com/GuillaumeGomez (65 contributions) (7 crates) (19 sponsors)
http://github.com/mitsuhiko (55 contributions) (2 crates) (3 sponsors)
http://github.com/yaahc (54 contributions) (2 crates) (32 sponsors)
http://github.com/djc (54 contributions) (2 crates) (5 sponsors)
http://github.com/taiki-e (36 contributions) (5 crates) (8 sponsors)
http://github.com/azriel91 (30 contributions) (2 crates) (3 sponsors)
http://github.com/jyn514 (27 contributions) (3 crates) (9 sponsors)
http://github.com/dalance (19 contributions) (1 crates) (4 sponsors)
http://github.com/palfrey (19 contributions) (1 crates) (0 sponsors)
http://github.com/Byron (15 contributions) (1 crates) (12 sponsors)
http://github.com/Milo123459 (15 contributions) (1 crates) (0 sponsors)
http://github.com/robinst (13 contributions) (2 crates) (0 sponsors)
http://github.com/lu-zero (12 contributions) (1 crates) (2 sponsors)
http://github.com/Mic92 (10 contributions) (1 crates) (4 sponsors)
http://github.com/daniellockyer (10 contributions) (3 crates) (0 sponsors)
http://github.com/jmmv (10 contributions) (1 crates) (0 sponsors)
http://github.com/CAD97 (10 contributions) (1 crates) (0 sponsors)
http://github.com/Swatinem (9 contributions) (3 crates) (3 sponsors)
...
```
