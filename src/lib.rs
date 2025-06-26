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
//! cargo add duat-hop@"*" --rename hop
//! ```
//!
//! # Usage
//!
//! In order to make use of it, just add the following to your `setup`
//! function:
//!
//! ```rust
//! # use duat_core::doc_duat as duat;
//! # use duat_hop as hop;
//! setup_duat!(setup);
//! use duat::prelude::*;
//! use hop::*;
//!
//! fn setup() {
//!     plug!(Hop);
//! }
//! ```
//!
//! When plugging this, the `w` key will be mapped to [`Hopper::word`]
//! in the [`User`] mode, while the `l` key will map onto
//! [`Hopper::line`] in the same mode.
//!
//! # Forms
//!
//! When plugging [`Hop`] this crate set the `"hop"` [`Form`] to
//! `"accent.info"`. This is then inherited by the following
//! [`Form`]s:
//!
//! - `"hop.one_char"` will be used on labels with just one character.
//! - `"hop.char1"` will be used on the first character of two
//!   character labels.
//! - `"hop.char2"` will be used on the secondcharacter of two
//!   character labels.
//!
//! Which you can modify via [`form::set`]:
//!
//! ```rust
//! # use duat_core::doc_duat as duat;
//! # use duat_hop as hop;
//! setup_duat!(setup);
//! use duat::prelude::*;
//! use hop::*;
//!
//! fn setup() {
//!     plug!(Hop);
//!
//!     form::set("hop.one_char", Form::red().underlined());
//!     form::set("hop.char1", "hop.one_char");
//!     form::set("hop.char2", "search
//! }
//! ```
//!
//! [`Mode`]: duat_core::mode::Mode
//! [`hop.nvim`]: https://github.com/smoka7/hop.nvim
//! [`User`]: duat_core::mode::User
//! [`Form`]: duat_core::form::Form
//! [`form::set`]: duat_core::form::set
use std::sync::LazyLock;

use duat_core::{prelude::*, text::Point};

/// The [`Plugin`] for the [`Hopper`] [`Mode`]
pub struct Hop;

impl<U: Ui> Plugin<U> for Hop {
    fn plug(self) {
        mode::map::<mode::User, U>("w", Hopper::word());
        mode::map::<mode::User, U>("l", Hopper::line());

        form::set_weak("hop", "accent.info");
    }
}

#[derive(Clone)]
pub struct Hopper {
    regex: &'static str,
    points: Vec<[Point; 2]>,
    seq: String,
}

impl Hopper {
    /// Returns a new instance of [`Hop`], moving by word by
    /// default
    pub fn word() -> Self {
        Self {
            regex: "[^\n\\s]+",
            points: Vec::new(),
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

impl<U: Ui> Mode<U> for Hopper {
    type Widget = File<U>;

    fn on_switch(&mut self, pa: &mut Pass, handle: Handle<File<U>, U>) {
        handle.write(pa, |file, _| {
            let cfg = file.print_cfg();
            let text = file.text_mut();

            let (start, _) = handle.area().start_points(text, cfg);
            let (end, _) = handle.area().end_points(text, cfg);

            self.points = text.search_fwd(self.regex, start..end).unwrap().collect();

            let seqs = key_seqs(self.points.len());

            for (seq, [p0, p1]) in seqs.iter().zip(&self.points) {
                let ghost = if seq.len() == 1 {
                    Ghost(txt!("[hop.one_char]{seq}"))
                } else {
                    let mut chars = seq.chars();
                    Ghost(txt!(
                        "[hop.char1]{}[hop.char2]{}",
                        chars.next().unwrap(),
                        chars.next().unwrap()
                    ))
                };

                text.insert_tag(*TAGGER, *p0, ghost);

                let seq_end = if p1.byte() == p0.byte() + 1
                    && let Some('\n') = text.char_at(*p1)
                {
                    p1.byte()
                } else {
                    let chars = text.strs(*p0..).chars().map(|c| c.len_utf8());
                    p0.byte() + chars.take(seq.len()).sum::<usize>()
                };

                text.insert_tag(*TAGGER, p0.byte()..seq_end, Conceal);
            }
        });
    }

    fn send_key(&mut self, pa: &mut Pass, key: KeyEvent, handle: Handle<File<U>, U>) {
        let char = match key {
            key!(KeyCode::Char(c)) => c,
            _ => {
                context::error!("Invalid label input");
                mode::reset::<File<U>, U>();
                return;
            }
        };

        self.seq.push(char);

        handle.write_selections(pa, |c| c.remove_extras());

        let seqs = key_seqs(self.points.len());
        for (seq, &[p0, p1]) in seqs.iter().zip(&self.points) {
            if *seq == self.seq {
                handle.edit_main(pa, |mut e| e.move_to(p0..p1));
                mode::reset::<File<U>, U>();
            } else if seq.starts_with(&self.seq) {
                continue;
            }
            // Removing one end of the conceal range will remove both ends.
            handle.write_text(pa, |text| text.remove_tags(*TAGGER, p0.byte()));
        }

        if self.seq.chars().count() == 2 || !LETTERS.contains(char) {
            mode::reset::<File<U>, U>();
        }
    }

    fn before_exit(&mut self, pa: &mut Pass, handle: Handle<Self::Widget, U>) {
        handle.write_text(pa, |text| text.remove_tags(*TAGGER, ..))
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
