//! A duat [`Mode`] to quickly move around the screen, inspired by
//! [`hop.nvim`].
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
//!     form::set("hop.one_char", Form::new().red().underlined());
//!     form::set("hop.char1", Form::mimic("hop.one_char"));
//!     form::set("hop.char2", Form::mimic("search"));
//! }
//! ```
//!
//! [`Mode`]: duat::mode::Mode
//! [`hop.nvim`]: https://github.com/smoka7/hop.nvim
//! [`User`]: duat::mode::User
//! [`Form`]: duat::form::Form
//! [`form::set`]: duat::form::set
use std::{ops::Range, sync::LazyLock};

use duat::{Plugin, Plugins, prelude::*};

/// The [`Plugin`] for the [`Hopper`] [`Mode`].
#[derive(Default)]
pub struct Hop;

impl Plugin for Hop {
    fn plug(self, opts: &mut Opts, _: &Plugins) {
        mode::map::<mode::User>("w", |pa: &mut Pass| mode::set(pa, Hopper::word()))
            .doc(txt!("[mode]Hop[] to a [a]word"));
        mode::map::<mode::User>("l", |pa: &mut Pass| mode::set(pa, Hopper::line()))
            .doc(txt!("[mode]Hop[] to a [a]line"));

        opts.whichkey.always_show::<Hopper>();

        form::set_weak("hop", Form::mimic("accent.info"));
        form::set_weak("hop.char2", Form::mimic("hop.char1"));

        let cloak_ns = Ns::new();

        hook::add::<ModeSwitched>(move |pa, mut switch| {
            if let Some(hop) = switch.new.get_as::<Hopper>() {
                let buffer = context::current_buffer(pa);

                let (buf, area) = buffer.write_with_area(pa);

                let opts = buf.print_opts();
                let mut text = buf.text_mut();

                let id = form::id_of!("cloak");
                text.insert_tag(cloak_ns, .., id.to_tag(239));

                let start = area.start_points(&text, opts).real;
                let end = area.end_points(&text, opts).real;

                hop.ranges = text.search(hop.regex).range(start..end).collect();

                let seqs = key_seqs(hop.ranges.len());

                for (seq, r) in seqs.iter().zip(&hop.ranges) {
                    let ghost = if seq.len() == 1 {
                        Overlay::new(txt!("[hop.one_char:240]{seq}"))
                    } else {
                        let mut chars = seq.chars();
                        Overlay::new(txt!(
                            "[hop.char1:240]{}[hop.char2:240]{}",
                            chars.next().unwrap(),
                            chars.next().unwrap()
                        ))
                    };

                    text.insert_tag(*NS, r.start, ghost);
                }
            } else if switch.old.is::<Hopper>() {
                let buffer = context::current_buffer(pa);

                let mut text = buffer.text_mut(pa);
                text.remove_tags(*NS, ..);
                text.remove_tags(cloak_ns, ..);
            }
        });
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
    fn bindings() -> mode::Bindings {
        mode::bindings!(match _ {
            unmod!(KeyCode::Char(..)) => txt!("Filter hopping entries"),
        })
    }

    fn send_key(&mut self, pa: &mut Pass, key_event: KeyEvent) {
        let char = match key_event {
            unmod!(KeyCode::Char(c)) => c,
            _ => {
                context::error!("Invalid label input");
                mode::reset::<Buffer>(pa);
                return;
            }
        };

        let buffer = context::current_buffer(pa);

        self.seq.push(char);

        buffer.write(pa).remove_extra_selections();

        let seqs = key_seqs(self.ranges.len());
        for (seq, r) in seqs.iter().zip(&self.ranges) {
            if *seq == self.seq {
                buffer.edit_main(pa, |mut e| e.move_to(r.clone()));
                mode::reset::<Buffer>(pa);
            } else if seq.starts_with(&self.seq) {
                continue;
            }
            // Removing one end of the conceal range will remove both ends.
            buffer.write(pa).text_mut().remove_tags(*NS, r.start);
        }

        if self.seq.chars().count() == 2 || !LETTERS.contains(char) {
            mode::reset::<Buffer>(pa);
        }
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
static NS: LazyLock<Ns> = Ns::new_lazy();
