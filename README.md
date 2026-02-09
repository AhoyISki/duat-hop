# duat-hop ![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue) [![duat-hop on crates.io](https://img.shields.io/crates/v/duat-hop)](https://crates.io/crates/duat-hop) [![duat-hop on docs.rs](https://docs.rs/duat-hop/badge.svg)](https://docs.rs/duat-hop) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/AhoyISki/duat-hop)

![](./assets/hop-demo.gif)

A duat [`Mode`][__link0] to quickly move around the screen, inspired by
[`hop.nvim`][__link1]

This plugin will highlight every word (or line, or a custom regex)
in the screen, and let you jump to it with at most 2 keypresses,
selecting the matched sequence.

## Installation

Just like other Duat plugins, this one can be installed by calling
`cargo add` in the config directory:

```bash
cargo add duat-hop@"*"
```

Or, if you are using a `--git-deps` version of duat, do this:

```bash
cargo add --git https://github.com/AhoyISki/duat-hop
```

## Usage

In order to make use of it, just add the following to your `setup`
function:

```rust
setup_duat!(setup);
use duat::prelude::*;

fn setup() {
    plug(duat_hop::Hop);
}
```

When plugging this, the `w` key will be mapped to [`Hopper::word`][__link2]
in the [`User`][__link3] mode, while the `l` key will map onto
[`Hopper::line`][__link4] in the same mode.

## Forms

When plugging [`Hop`][__link5] will set the `"hop"` [`Form`][__link6] to
`"accent.info"`. This is then inherited by the following
[`Form`][__link7]s:

* `"hop.one_char"` will be used on labels with just one character.
* `"hop.char1"` will be used on the first character of two
  character labels.
* `"hop.char2"` will be used on the second character of two
  character labels. By default, this form inherits `"hop.char1"`.

Which you can modify via [`form::set`][__link8]:

```rust
setup_duat!(setup);
use duat::prelude::*;

fn setup() {
    plug(duat_hop::Hop);

    form::set("hop.one_char", Form::new().red().underlined());
    form::set("hop.char1", Form::mimic("hop.one_char"));
    form::set("hop.char2", Form::mimic("search"));
}
```


 [__cargo_doc2readme_dependencies_info]: ggGkYW0BYXSEG3foRquKx5scG8C5J9KGegetGzgLv6SF_cZyG5uY3D4-wulKYXKEGzldXkre1TcnG0XSdiXIsA8kG2GlwOVvKBSeG-sQxhSA_8-TYWSCgmRkdWF0ZTAuOC4zg2hkdWF0LWhvcGUwLjIuMmhkdWF0X2hvcA
 [__link0]: https://docs.rs/duat/0.8.3/duat/?search=mode::Mode
 [__link1]: https://github.com/smoka7/hop.nvim
 [__link2]: https://docs.rs/duat-hop/0.2.2/duat_hop/?search=Hopper::word
 [__link3]: https://docs.rs/duat/0.8.3/duat/?search=mode::User
 [__link4]: https://docs.rs/duat-hop/0.2.2/duat_hop/?search=Hopper::line
 [__link5]: https://docs.rs/duat-hop/0.2.2/duat_hop/struct.Hop.html
 [__link6]: https://docs.rs/duat/0.8.3/duat/?search=form::Form
 [__link7]: https://docs.rs/duat/0.8.3/duat/?search=form::Form
 [__link8]: https://docs.rs/duat/0.8.3/duat/?search=form::set
