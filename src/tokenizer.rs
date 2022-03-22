use crate::tag::Tag;
use crate::BLOCK_QUOTE_TAG;
use crate::HEADER_TAG;
use chr::Chr;
use chr::ChrIter;
use std::io;
use std::io::Read;
use std::iter::Peekable;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    // just 1 ~ 6
    Space,
    Tab,
    Ln,
    Tag(Tag),
    PureText { data: &'a [u8], char_len: u32 },
}

impl std::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => write!(f, "Space"),
            Self::Tab => write!(f, "Tab"),
            Self::Ln => write!(f, "LineFeed"),
            Self::Tag(arg0) => f.debug_tuple("Tag").field(arg0).finish(),
            Self::PureText {
                data: arg0,
                char_len: _len,
            } => {
                write!(f, "{:?}", std::str::from_utf8(arg0))
            }
        }
    }
}

impl<'a> From<&'a Chr> for Token<'a> {
    fn from(c: &'a Chr) -> Self {
        Token::PureText {
            data: c.into(),
            char_len: 1,
        }
    }
}

impl<'a> From<&'a CharCache> for Token<'a> {
    fn from(cache: &'a CharCache) -> Self {
        debug_assert!(cache.has_element());

        Token::PureText {
            data: cache.get(),
            char_len: cache.char_len,
        }
    }
}

pub struct Tokenizer<'a, R: Read> {
    iter: Peekable<ChrIter<R>>,
    state: State,
    cache: CharCache,
    temp_chr: Chr,
    /* from above temp_chr OR cache */
    _marker: PhantomData<&'a [u8]>,
}

#[derive(Clone, Copy)]
enum State {
    Start,
    LineFeed,
    Tab(u32),
    Other,
}

// max = 6 # + 4-byte u8 char
const ARRAY_CAP: usize = 7 + 3;
#[derive(Default)]
struct CharCache {
    arr: [u8; ARRAY_CAP],
    len: usize,
    char_len: u32,
}

impl CharCache {
    fn has_element(&self) -> bool {
        self.len != 0
    }

    fn put(&mut self, u: u8) {
        debug_assert_ne!(self.len, ARRAY_CAP);

        unsafe {
            *self.arr.get_unchecked_mut(self.len) = u;
        }
        self.len += 1;
        self.char_len += 1;
    }

    fn put_chr(&mut self, chr: Chr) {
        for &b in <&Chr as Into<&[u8]>>::into(&chr) {
            self.put(b);
        }
        self.char_len += 1;
    }

    fn get(&self) -> &[u8] {
        &self.arr[..self.len]
    }

    fn clear(&mut self) {
        self.len = 0;
        self.char_len = 0;
    }
}

impl<'a, R: Read + 'a> Tokenizer<'a, R> {
    pub fn new(r: R) -> Self {
        Self {
            iter: ChrIter::new(r).peekable(),
            cache: CharCache::default(),
            state: State::Start,
            _marker: PhantomData,
            temp_chr: Chr::One([b' ']),
        }
    }

    fn get_lifetime(&self) -> &'a Self {
        unsafe { &*(self as *const Self) }
    }

    fn get_pure_text(&mut self, chr: Chr) -> Token<'a> {
        self.temp_chr = chr;
        (&self.get_lifetime().temp_chr).into()
    }

    fn get_cache(&mut self) -> Option<io::Result<Token<'a>>> {
        let ret = Some(Ok((&self.get_lifetime().cache).into()));
        self.cache.clear();
        ret
    }
}

impl<'a, R: Read + 'a> Iterator for Tokenizer<'a, R> {
    type Item = io::Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.iter.next()? {
                Ok(chr) => {
                    return Some(Ok(if let Chr::One(b) = chr {
                        match unsafe { *b.get_unchecked(0) } {
                            b'\\' => match self.iter.peek() {
                                Some(res) => match res {
                                    Ok(chr) => {
                                        let chr = *chr;
                                        self.iter.next();
                                        self.state = State::Other;

                                        self.get_pure_text(chr)
                                    }
                                    Err(_e) => continue 'outer,
                                },
                                None => return None,
                            },
                            b' ' => {
                                self.state = State::Other;

                                Token::Space
                            }
                            b'\t' => {
                                self.state = State::Tab(if let State::Tab(n) = self.state {
                                    n + 1
                                } else {
                                    1
                                });

                                Token::Tab
                            }
                            b'\n' => {
                                self.state = State::LineFeed;

                                Token::Ln
                            }
                            b'\r' => match self.iter.peek() {
                                Some(e) => match e {
                                    Ok(u) => {
                                        if u == &b'\n' {
                                            self.iter.next();
                                        }
                                        self.state = State::LineFeed;

                                        Token::Ln
                                    }

                                    Err(_e) => continue,
                                },
                                None => Token::Ln,
                            },
                            b => {
                                let last_state = self.state;
                                self.state = State::Other;

                                match last_state {
                                    // 如果上个状态是换行或Start，才进行Tag的解析，方便后续的Parser的操作
                                    State::LineFeed | State::Start => match b {
                                        crate::HEADER_TAG => {
                                            let mut head_level = 1;
                                            self.cache.put(HEADER_TAG);

                                            while let Some(res) = self.iter.next() {
                                                match res {
                                                    Ok(c) => {
                                                        self.cache.put_chr(c);

                                                        if c == HEADER_TAG {
                                                            head_level += 1;

                                                            // 如果有7个#了，就把这7个#当做纯文本对待
                                                            if head_level == 7 {
                                                                return self.get_cache();
                                                            }
                                                        } else if c == b' ' {
                                                            break;
                                                        } else {
                                                            return self.get_cache();
                                                        }
                                                    }
                                                    Err(_e) => continue 'outer,
                                                }
                                            }

                                            self.cache.clear();
                                            Token::Tag(Tag::Header(head_level))
                                        }
                                        crate::BLOCK_QUOTE_TAG => match self.iter.next() {
                                            Some(e) => match e {
                                                Ok(c) => {
                                                    if c == b' ' {
                                                        Token::Tag(Tag::BlockQuote)
                                                    } else {
                                                        self.cache.put(BLOCK_QUOTE_TAG);
                                                        self.cache.put_chr(c);

                                                        return self.get_cache();
                                                    }
                                                }
                                                Err(_e) => continue,
                                            },
                                            None => self.get_pure_text(BLOCK_QUOTE_TAG.into()),
                                        },
                                        b'-' => {
                                            let mut is_done_task = false;
                                            self.cache.put(b'-');

                                            for mut counter in [b' ', b'[', b'?', b']', b' '] {
                                                match self.iter.next() {
                                                    Some(res) => match res {
                                                        Ok(cur_chr) => {
                                                            if counter == b'?' {
                                                                match cur_chr {
                                                                    Chr::One([b' ']) => {
                                                                        /* fall down */
                                                                        counter = b' ';
                                                                    }
                                                                    Chr::One([b'x']) => {
                                                                        counter = b'x';
                                                                        is_done_task = true;
                                                                    }
                                                                    _ => {
                                                                        self.cache.put_chr(cur_chr);
                                                                        return self.get_cache();
                                                                    }
                                                                }
                                                            } else {
                                                                #[allow(
                                                                    clippy::collapsible_else_if
                                                                )]
                                                                if cur_chr != counter {
                                                                    self.cache.put_chr(cur_chr);
                                                                    return self.get_cache();
                                                                }
                                                            }

                                                            self.cache.put(counter);
                                                        }
                                                        Err(_e) => continue 'outer,
                                                    },
                                                    None => continue 'outer,
                                                }
                                            }

                                            // must clear, because it means all match!
                                            self.cache.clear();
                                            Token::Tag(Tag::TaskList(is_done_task))
                                        }

                                        _ => self.get_pure_text(chr),
                                    },
                                    _ => self.get_pure_text(chr),
                                }
                            }
                        }
                    } else {
                        self.get_pure_text(chr)
                    }));
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl Token<'_> {
    pub fn char_len(self) -> u32 {
        match self {
            Token::Space => 1,
            Token::Tab => 1,
            Token::Ln => 0,
            Token::PureText { data, char_len } => char_len,
            Token::Tag(tag) => tag.char_len(),
        }
    }
}

#[test]
fn test() {
    let tokenizer = Tokenizer::new(std::fs::File::open("test.md").unwrap());
    for t in tokenizer {
        println!("{:?}", t);
    }
}
