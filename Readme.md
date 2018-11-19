<p align="center"><img width="100" src="https://i.imgur.com/3GfOkqo.png" alt="torchbear logo"><br>This Project is Currently in Stealth Mode.<br>please do not post a news story until v1 is released very shortly.<br>thank you.</p>

<p align="center">
  <a href="https://www.travis-ci.com/foundpatterns/torchbear"><img src="https://travis-ci.com/foundpatterns/torchbear.svg?branch=master" alt="Travis Build Status"></a>
  <a href="https://ci.appveyor.com/project/mitchtbaum/torchbear"><img src="https://ci.appveyor.com/api/projects/status/mg6e0p7s5v7j61ja?svg=true" alt="Appveyor Build Status"></a>
  <a href="https://deps.rs/crate/torchbear/0.5.0"><img src="https://deps.rs/crate/torchbear/0.5.0/status.svg" alt="Dependencies"></a>
  <a href="https://crates.io/crates/torchbear"><img src="https://img.shields.io/crates/v/torchbear.svg" alt="torchbear Crate"></a>
  <a href="https://github.com/foundpatterns/torchbear/releases"><img src="https://img.shields.io/github/downloads/foundpatterns/torchbear/total.svg" alt="Download Total"></a>
  <br>
  <a href="https://github.com/rust-lang/crates.io/issues/704"><img src="https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg" alt="Actively Maintained"></a>
  <a href="https://opensource.com/life/16/1/8-ways-contribute-open-source-without-writing-code"><img src="https://img.shields.io/badge/contributions-welcome-brightgreen.svg?style=" alt="Contributions Welcome"></a>
  <a href="https://akrabat.com/the-beginners-guide-to-contributing-to-a-github-project/#to-sum-up"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs Welcome"></a>
  <a href="https://en.wikipedia.org/wiki/List_of_parties_to_international_copyright_agreements"><img src="https://img.shields.io/badge/License-MIT%2FApache2-blue.svg" alt="License: MIT/Apache"></a>
  <a href="https://discord.gg/sWCQxT"><img src="https://img.shields.io/badge/chat-on%20discord-7289da.svg" alt="Chat"></a>
</p>

Torchbear is an extremely fast and featureful Lua application framework.  It gives you power of Rust with the simplicity of Lua.  You can use it for web automation, embedded programming, and anything else you can imagine.

## Built-in Tools

* [rlua](https://github.com/kyren/rlua) safely wrapped Lua 5.3.5 with traceback error messages
* [Actix Web](https://github.com/actix/actix-web) HTTP/1.x and HTTP/2.0 web servers and clients
* [Tera](https://github.com/Keats/tera) Jinja template rendering
* [Comrak](https://github.com/kivikakk/comrak) Markdown output
* [Libsodium](https://github.com/maidsafe/rust_sodium) cryptographic signing and verifying, and encrypting and decrypting
* [`std::fs::*`](https://doc.rust-lang.org/std/fs/index.html) filesystem operations
* [`std::collections::HashSet`](https://doc.rust-lang.org/std/collections/struct.HashSet.html) set-theoretic operations
* [Serde](https://github.com/serde-rs/serde) YAML and JSON serializing/deserializing
* [UUID-rs](https://github.com/uuid-rs/uuid) UUID generation and verification
* [Chrono](https://github.com/chronotope/chrono) time/date generation and verification
* [Select-rs](https://github.com/utkarshkukreti/select.rs) HTML scraping
* [Git](https://github.com/alexcrichton/git2-rs) repo creation, commit staging, and log access
* [Tantivy](https://github.com/tantivy-search/tantivy) schema building, document adding/updating/deleting, and searching
* [regex](https://github.com/rust-lang/regex) matching and replacing

## Installation

Torchbear comes as a single executable, making it very easy to install.  The latest version is available on [Torchbear's GitHub releases page](https://github.com/foundpatterns/torchbear/releases), so you can download the zip file for your operating system and hardware architecture from there and unzip the executable wherever is most convenient for you.  To make it simpler, you can use our install script by copying and pasting this line into your terminal, then just running `torchbear` or `torchbear.exe` in any of your apps:

```sh
 curl https://git.io/fpcV6 -sSfL | sh
 ```

#### How to add a terminal on non-linux environments

(If you haven't heard of a terminal before, here's a [1 min intro to what is a terminal window](https://www.youtube.com/watch?v=zw7Nd67_aFw).)

Windows: install [Cmder](http://cmder.net/) Full.

Android users, install [Termux](https://termux.com/).

MacOS comes ready, but [Homebrew](https://brew.sh/) has many additional tools.

## Examples

#### Hello World App

- in `init.lua`

`print("hello from Torchbear")`

- in `Settings.toml`

`init = "init.lua"`

- run `torchbear`

#### [Torchbear Static Webserver](https://github.com/foundpatterns/torchbear-static-webserver)

#### [Lighttouch Application Framework](https://github.com/foundpatterns/lighttouch) (👍👍 for Web Development!)

## Contributions wanted

Torchbear extends Rust's burgeoning ecosystem of libraries. Developers are welcomed to [make small changes](https://github.com/foundpatterns/torchbear/issues?q=is%3Aopen+is%3Aissue+label%3Asize%2F0.25) as well as high impact contributions, like [adding bindings](https://github.com/foundpatterns/torchbear/labels/feature%2Fbindings).  There are many examples to learn from in the bindings directory, each with an interesting history.  You'll learn a Rust library's API inside and out, and you'll put another tool into the hands of a growing userbase.
