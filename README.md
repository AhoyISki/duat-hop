# duat-hop ![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue) [![duat-hop on crates.io](https://img.shields.io/crates/v/duat-hop)](https://crates.io/crates/duat-hop) [![duat-hop on docs.rs](https://docs.rs/duat-hop/badge.svg)](https://docs.rs/duat-hop) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/AhoyISki/duat-hop)

![hop demonstration](./assets/hop-demonstration.gif)

A duat [`Mode`][__link0] to quickly move around the screen, inspired by
[`hop.nvim`][__link1]

This plugin will highlight every word (or line, or a custom regex)
in the screen, and let you jump to it with at most 2 keypresses,
selecting the matched sequence.

## Installation

Just like other Duat plugins, this one can be installed by calling
`cargo add` in the config directory:

```bash
cargo add duat-hop@"*" --rename hop
```

## Usage

In order to make use of it, just add the following to your `setup`
function:

```rust
setup_duat!(setup);
use duat::prelude::*;
use hop::*;

fn setup() {
    plug!(Hop);
}
```

When plugging this, the `w` key will be mapped to [`Hopper::word`][__link2]
in the [`User`][__link3] mode, while the `l` key will map onto
[`Hopper::line`][__link4] in the same mode.

## Forms

When plugging [`Hop`][__link5] this crate set the `"hop"` [`Form`][__link6] to
`"accent.info"`. This is then inherited by the following
[`Form`][__link7]s:

* `"hop.one_char"` will be used on labels with just one character.
* `"hop.char1"` will be used on the first character of two
  character labels.
* `"hop.char2"` will be used on the secondcharacter of two
  character labels.

Which you can modify via [`form::set`][__link8]:

```rust
setup_duat!(setup);
use duat::prelude::*;
use hop::*;

fn setup() {
    plug!(Hop);

    form::set("hop.one_char", Form::red().underlined());
    form::set("hop.char1", "hop.one_char");
    form::set("hop.char2", "search
}
```


 [__cargo_doc2readme_dependencies_info]: ggGkYW0BYXSEG0NWJwoPKLNnG0TAj5IG93PSG7YmxsASqSxYGzkEnHTUwH4sYXKEG8Ork8UsEunVG10b8XOUE4_0G39SETzS8DVdG1hUsh1A52CfYWSCg2hkdWF0LWhvcGUwLjEuMGhkdWF0X2hvcIJpZHVhdF9jb3JlZTAuNS4z
 [__link0]: https://docs.rs/duat_core/0.5.3/duat_core/?search=mode::Mode
 [__link1]: https://github.com/smoka7/hop.nvim
 [__link2]: https://docs.rs/duat-hop/0.1.0/duat_hop/?search=Hopper::word
 [__link3]: https://docs.rs/duat_core/0.5.3/duat_core/?search=mode::User
 [__link4]: https://docs.rs/duat-hop/0.1.0/duat_hop/?search=Hopper::line
 [__link5]: https://docs.rs/duat-hop/0.1.0/duat_hop/struct.Hop.html
 [__link6]: https://docs.rs/duat_core/0.5.3/duat_core/?search=form::Form
 [__link7]: https://docs.rs/duat_core/0.5.3/duat_core/?search=form::Form
 [__link8]: https://docs.rs/duat_core/0.5.3/duat_core/?search=form::set
