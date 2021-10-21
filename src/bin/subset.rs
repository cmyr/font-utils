//! A tool for creating a new ufo from a subset of an existing ufo's glyphs.
//!
//! This also drops groups, kerning and features.
//!
//! (the sole purpose of this script is generating fonts for testing
//! feature compilation.)

use std::{collections::HashSet, env, ffi::OsStr, path::PathBuf};

fn main() {
    let args = Args::get_from_env_or_exit();
    let mut ufo = norad::Font::load(&args.path).unwrap();
    // Prune all non-foreground layers.
    let default_layer_name = ufo.layers.default_layer().name().clone();
    let to_remove: Vec<_> = ufo
        .layers
        .names()
        .filter(|l| *l != &default_layer_name)
        .cloned()
        .collect();
    for layer_name in to_remove {
        ufo.layers.remove(&layer_name);
    }

    // remove any kerning/groups
    ufo.kerning = None;
    ufo.groups = None;
    ufo.features = None;

    let keep_names: HashSet<_> = NAMES_TO_KEEP.iter().cloned().collect();
    let mut to_keep = HashSet::new();

    for glyph in ufo.default_layer().iter() {
        let keep = matches!(glyph.codepoints.first(), Some('\0'..='~'))
            || keep_names.contains(&*glyph.name);
        if keep {
            to_keep.insert(glyph.name.clone());
            to_keep.extend(glyph.components.iter().map(|c| c.base.clone()));
        }
    }

    let to_delete = ufo
        .default_layer()
        .iter()
        .filter_map(|g| {
            if to_keep.contains(&g.name) {
                None
            } else {
                Some(g.name.clone())
            }
        })
        .collect::<HashSet<_>>();

    for glyph in &to_delete {
        ufo.default_layer_mut().remove_glyph(&glyph);
    }

    if let Some(order) = ufo
        .lib
        .get_mut("public.glyphOrder")
        .and_then(|v| v.as_array_mut())
    {
        order.retain(|t| to_keep.contains(t.as_string().unwrap()));
    }

    if let Some(names) = ufo
        .lib
        .get_mut("public.postscriptNames")
        .and_then(|v| v.as_dictionary_mut())
    {
        names.retain(|key, _| to_keep.contains(key.as_str()));
    }

    ufo.meta.creator = "org.linebender.norad".to_string();
    if let Err(e) = ufo.save(args.outpath) {
        eprintln!("Saving UFO failed: {}", e);
        std::process::exit(1);
    }
}

struct Args {
    path: PathBuf,
    outpath: PathBuf,
}

macro_rules! exit_err {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        std::process::exit(1);
    })
}

impl Args {
    fn get_from_env_or_exit() -> Self {
        let mut args = env::args().skip(1);
        let path = match args.next().map(PathBuf::from) {
            Some(ref p) if p.exists() && p.extension() == Some(OsStr::new("ufo")) => p.to_owned(),
            Some(ref p) => exit_err!("path {:?} is not an existing .ufo file, exiting", p),
            None => exit_err!("Please supply a path to a .ufo file"),
        };

        let outpath = match args.next().map(PathBuf::from) {
            Some(path) if path.exists() => {
                exit_err!("outpath {} already exists, exiting", path.display())
            }
            None => exit_err!("please supply a destination path"),
            Some(other) => other,
        };

        Args { path, outpath }
    }
}
static NAMES_TO_KEEP: &[&str] = &[
    "eacute",
    "egrave",
    ".notdef",
    "f_f",
    "f_i",
    "f_f_i",
    "g.salt",
    "zero.slash",
    "zero.osf",
    "one.osf",
    "two.osf",
    "three.osf",
    "four.osf",
    "five.osf",
    "six.osf",
    "seven.osf",
    "eight.osf",
    "nine.osf",
];
