//! A duat [`Mode`] to quickly move around the screen, inspired by
//! [`hop.nvim`]
//!
//! This plugin will highlight every word (or line, or a custom regex)
//! in the screen, and let you jump to it with at most 2 keypresses,
//! selecting the matched sequence.
//!
//! # Installation
//!
//! Just like other Duat plugins, this one can be installed by calling
//! `cargo add` in the config directory:
//!
//! ```bash
//! cargo add duat-hop@"*"
//! ```
//!
//! Or, if you are using a `--git-deps` version of duat, do this:
//!
//! ```bash
//! cargo add --git https://github.com/AhoyISki/duat-hop
//! ```
//!
//! # Usage
//!
//! In order to make use of it, just add the following to your `setup`
//! function:
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//!
//! fn setup() {
//!     plug(duat_hop::Hop);
//! }
//! ```
//!
//! When plugging this, the `w` key will be mapped to [`Hopper::word`]
//! in the [`User`] mode, while the `l` key will map onto
//! [`Hopper::line`] in the same mode.
//!
//! # Forms
//!
//! When plugging [`Hop`] will set the `"hop"` [`Form`] to
//! `"accent.info"`. This is then inherited by the following
//! [`Form`]s:
//!
//! - `"hop.one_char"` will be used on labels with just one character.
//! - `"hop.char1"` will be used on the first character of two
//!   character labels.
//! - `"hop.char2"` will be used on the second character of two
//!   character labels. By default, this form inherits `"hop.char1"`.
//!
//! Which you can modify via [`form::set`]:
//!
//! ```rust
//! setup_duat!(setup);
//! use duat::prelude::*;
//!
//! fn setup() {
//!     plug(duat_hop::Hop);
//!
//!     form::set("hop.one_char", Form::red().underlined());
//!     form::set("hop.char1", "hop.one_char");
//!     form::set("hop.char2", "search");
//! }
//! ```
//!
//! [`Mode`]: duat_core::mode::Mode
//! [`hop.nvim`]: https://github.com/smoka7/hop.nvim
//! [`User`]: duat_core::mode::User
//! [`Form`]: duat_core::form::Form
//! [`form::set`]: duat_core::form::set
use std::{ops::Range, sync::LazyLock};

use duat::prelude::*;

/// The [`Plugin`] for the [`Hopper`] [`Mode`]
#[derive(Default)]
pub struct Hop;

impl Plugin for Hop {
    fn plug(self, _: &Plugins) {
        mode::map::<mode::User>("w", Hopper::word());
        mode::map::<mode::User>("l", Hopper::line());

        form::set_weak("hop", "accent.info");
        form::set_weak("hop.char2", "hop.char1");
    }
}

#[derive(Clone)]
pub struct Hopper {
    regex: &'static str,
    ranges: Vec<Range<usize>>,
    seq: String,
}

impl Hopper {
    /// Returns a new instance of [`Hop`], moving by word by
    /// default
    pub fn word() -> Self {
        Self {
            regex: "[^\n\\s]+",
            ranges: Vec::new(),
            seq: String::new(),
        }
    }

    /// Changes this [`Mode`] to move by line, not by word
    pub fn line() -> Self {
        Self { regex: "[^\n\\s][^\n]+", ..Self::word() }
    }

    /// Use a custom regex instead of the word or line regexes
    pub fn with_regex(regex: &'static str) -> Self {
        Self { regex, ..Self::word() }
    }
}

impl Mode for Hopper {
    type Widget = Buffer;

    fn on_switch(&mut self, pa: &mut Pass, handle: Handle) {
        let (file, area) = handle.write_with_area(pa);

        let opts = file.opts;
        let text = file.text_mut();

        let id = form::id_of!("cloak");
        text.insert_tag(*CLOAK_TAGGER, .., id.to_tag(101));

        let start = area.start_points(text, opts).real;
        let end = area.end_points(text, opts).real;

        self.ranges = text.search_fwd(self.regex, start..end).unwrap().collect();

        let seqs = key_seqs(self.ranges.len());

        for (seq, r) in seqs.iter().zip(&self.ranges) {
            let ghost = if seq.len() == 1 {
                Ghost(txt!("[hop.one_char:102]{seq}"))
            } else {
                let mut chars = seq.chars();
                Ghost(txt!(
                    "[hop.char1:102]{}[hop.char2:102]{}",
                    chars.next().unwrap(),
                    chars.next().unwrap()
                ))
            };

            text.insert_tag(*TAGGER, r.start, ghost);

            let seq_end = if r.end == r.start + 1
                && let Some('\n') = text.char_at(r.end)
            {
                r.end
            } else {
                let chars = text.strs(r.start..).unwrap().chars().map(|c| c.len_utf8());
                r.start + chars.take(seq.len()).sum::<usize>()
            };

            text.insert_tag(*TAGGER, r.start..seq_end, Conceal);
        }
    }

    fn send_key(&mut self, pa: &mut Pass, key_event: KeyEvent, handle: Handle) {
        let char = match key_event {
            event!(KeyCode::Char(c)) => c,
            _ => {
                context::error!("Invalid label input");
                mode::reset::<Buffer>();
                return;
            }
        };

        self.seq.push(char);

        handle.write(pa).selections_mut().remove_extras();

        let seqs = key_seqs(self.ranges.len());
        for (seq, r) in seqs.iter().zip(&self.ranges) {
            if *seq == self.seq {
                handle.edit_main(pa, |mut e| e.move_to(r.clone()));
                mode::reset::<Buffer>();
            } else if seq.starts_with(&self.seq) {
                continue;
            }
            // Removing one end of the conceal range will remove both ends.
            handle.write(pa).text_mut().remove_tags(*TAGGER, r.start);
        }

        if self.seq.chars().count() == 2 || !LETTERS.contains(char) {
            mode::reset::<Buffer>();
        }
    }

    fn before_exit(&mut self, pa: &mut Pass, handle: Handle<Self::Widget>) {
        handle
            .write(pa)
            .text_mut()
            .remove_tags([*TAGGER, *CLOAK_TAGGER], ..)
    }
}

fn key_seqs(len: usize) -> Vec<String> {
    let double = len / LETTERS.len();
    let mut seqs = Vec::new();

    seqs.extend(LETTERS.chars().skip(double).map(char::into));
    let chars = LETTERS.chars().take(double);
    seqs.extend(chars.flat_map(|c1| LETTERS.chars().map(move |c2| format!("{c1}{c2}"))));

    seqs
}

static LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
static TAGGER: LazyLock<Tagger> = Tagger::new_static();
static CLOAK_TAGGER: LazyLock<Tagger> = Tagger::new_static();
