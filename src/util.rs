use {
    crate::{
        ast::Name,
    },
    derive_more::Display,
    std::fmt::{self, Display},
};

pub fn unescape(s: &str) -> String {
    let mut escaping = false;
    let mut output = String::new();

    for c in s.chars() {
        if !escaping && c == '\\' {
            escaping = true;
            continue
        }

        let escaped = if escaping {
            match c {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => c,
            }
        } else {
            c
        };

        output.push(escaped);
        escaping = false;
        continue
    }

    output
}

pub fn join<I: Iterator<Item=impl Display> + Clone>(
    joiner: &'static str,
    it: I,
) -> impl Display {
    struct Join<I>(&'static str, I);

    impl<I: Iterator<Item=impl Display> + Clone> Display for Join<I> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Join(joiner, it) = self;
            let mut it: I = it.clone();

            match it.next() {
                Some(d) => write!(fmt, "{}", d)?,
                None => return Ok(()),
            }

            for d in it {
                write!(fmt, "{}{}", joiner, d)?;
            }

            Ok(())
        }
    }

    Join(joiner, it)
}

#[derive(Display)]
#[display(fmt = "{}{}{}", key, between, value)]
pub struct Mapping<K: Display, V: Display> {
    between: &'static str,
    key: K,
    value: V,
}

pub fn mapping<K: Display, V: Display>(
    between: &'static str,
) -> impl Fn((K, V)) -> Mapping<K, V> + Copy {
    move |(key, value)| Mapping {between, key, value}
}

pub type Map<K, V> = fnv::FnvHashMap<K, V>;
