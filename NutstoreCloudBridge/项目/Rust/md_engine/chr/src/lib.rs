use std::io::{self, Read};

const BUF_SIZE: usize = 8 * 1024;
// const BUF_SIZE: usize = 24;
// const BUF_SIZE: usize = 7;

pub struct ChrIter<R> {
    buf: [u8; BUF_SIZE],
    r: R,
    cursor: usize,
    buf_len: usize,
    remain: Remain,
}

// 这个struct呢就是记录buf末尾剩余的byte的，因为最多只可能有3个byte
#[derive(Clone, Copy)]
struct Remain {
    head_len: u8,
    tail_len: u8,
    a: u8,
    b: u8,
    c: u8,
}

#[derive(Clone, Copy, PartialEq, Eq,Debug)]
pub enum Chr {
    One([u8; 1]),
    Two([u8; 2]),
    Three([u8; 3]),
    Four([u8; 4]),
}



impl PartialEq<u8> for Chr {
    fn eq(&self, other: &u8) -> bool {
		unsafe {
        if let Self::One(b) = self {
            b.get_unchecked(0) == other
        } else {
            false
        }
   } }
}

impl From<u8> for Chr {
    fn from(b: u8) -> Self {
		Chr::One([b])
    }
}

impl<'a> From<&'a Chr> for &'a [u8] {
    fn from(c: &'a Chr) -> Self {
		 match c {
            Chr::One(x) => &x[..],
            Chr::Two(x) => &x[..],
            Chr::Three(x) => &x[..],
            Chr::Four(x) => &x[..],
        }
    }
}

impl Remain {
    fn new() -> Self {
        Self {
            head_len: 0,
            tail_len: 0,
            a: 0,
            b: 0,
            c: 0,
        }
    }

    fn is_empty(self) -> bool {
        self.head_len == 0
    }

    fn clear(&mut self) {
        self.head_len = 0
    }
}

impl<R> ChrIter<R> {
    pub fn new(r: R) -> Self {
        Self {
            buf: [0; BUF_SIZE],
            buf_len: usize::MAX,
            r,
            cursor: 0,
            remain: Remain::new(),
        }
    }

    fn cur_u8_unite_len(&self) -> (u8, u8) {
        let a = self.get_byte(self.cursor);
        (utf8_len(a), a)
    }

    fn set_load_buf(&mut self) {
        // 此时如果当前cursor已经等于buf.len()，那么我们再次read到buf一次~
        self.buf_len = usize::MAX;
        self.cursor = 0;
    }

    // 假定cursor和buf在一个合理范围内，同时必定buf是能够截取到对应长度的
    // 然后我们写一个utf8 -> unicode的转换函数
    fn chr(&mut self, char_len: u8, a: u8) -> Chr {
        let chr = match char_len {
            1 => Chr::One([a]),
            2 => Chr::Two([a, self.get_byte(self.cursor + 1)]),
            3 => Chr::Three([
                a,
                self.get_byte(self.cursor + 1),
                self.get_byte(self.cursor + 2),
            ]),
            4 => Chr::Four([
                a,
                self.get_byte(self.cursor + 1),
                self.get_byte(self.cursor + 2),
                self.get_byte(self.cursor + 3),
            ]),
            // TODO change unreachable!() to unsafe uncheck unreachable
            _ => unreachable!(),
        };

        self.cursor += char_len as usize;
        chr
    }

    fn get_byte(&self, index: usize) -> u8 {
        unsafe { *self.buf.get_unchecked(index) }
    }

    fn set_remain(&mut self, a: u8, head_len: u8, tail_len: u8) {
        self.remain = match head_len {
            1 => Remain {
                head_len,
                tail_len,
                a,
                b: 0,
                c: 0,
            },
            2 => Remain {
                head_len,
                tail_len,
                a,
                b: self.get_byte(self.cursor + 1),
                c: 0,
            },
            3 => Remain {
                head_len,
                tail_len,
                a,
                b: self.get_byte(self.cursor + 1),
                c: self.get_byte(self.cursor + 2),
            },
            _ => unreachable!(),
        };
    }

    fn get_remain_unite(&mut self) -> Chr {
        let ret = match self.remain.head_len {
            1 => match self.remain.tail_len {
                1 => Chr::Two([self.remain.a, self.get_byte(self.cursor)]),
                2 => Chr::Three([
                    self.remain.a,
                    self.get_byte(self.cursor),
                    self.get_byte(self.cursor + 1),
                ]),
                3 => Chr::Four([
                    self.remain.a,
                    self.get_byte(self.cursor),
                    self.get_byte(self.cursor + 1),
                    self.get_byte(self.cursor + 2),
                ]),
                _ => unreachable!(),
            },
            2 => match self.remain.tail_len {
                1 => Chr::Three([self.remain.a, self.remain.b, self.get_byte(self.cursor)]),
                2 => Chr::Four([
                    self.remain.a,
                    self.remain.b,
                    self.get_byte(self.cursor),
                    self.get_byte(self.cursor + 1),
				]),
                _ => unreachable!(),
            },
            3 => match self.remain.tail_len {
                1 => Chr::Four([
                    self.remain.a,
                    self.remain.b,
                    self.remain.c,
                    self.get_byte(self.cursor),
                ]),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        self.cursor += self.remain.tail_len as usize;
        self.remain.clear();
        ret
    }
}

impl<R: Read> Iterator for ChrIter<R> {
    type Item = io::Result<Chr>;

    fn next(&mut self) -> Option<Self::Item> {
        // 那么我们先去判断这个len，大概有三种情况，初始数值、等于buf.len()、小于buf.len()
        // 我们还需要一个cursor去表明当前的buf的位置！
        loop {
            if self.buf_len == usize::MAX {
                // println!("reach the MAX!");
                // 每次在这个节点时，需要判断下上个remain是否不为空，即存在剩余
                // byte，我们把它拼接起来。但还是先写构造时，清楚是如何构造的
                self.buf_len = match self.r.read(&mut self.buf) {
                    Ok(l) => l,
                    Err(e) => return Some(Err(e)),
                };

                if self.buf_len == 0 {
                    return None;
                }

                if !self.remain.is_empty() {
                    return Some(Ok(self.get_remain_unite()));
                }

                let (len, a) = self.cur_u8_unite_len();

                return Some(Ok(self.chr(len, a)));
            } else if self.buf_len == self.buf.len() {
                // println!("reach the eq!");
                // 此时在判断self.cursor和buf.len()的关系~
                if self.cursor < self.buf_len {
                    let (char_len, a) = self.cur_u8_unite_len();
                    let tail_len = char_len as i32 + self.cursor as i32 - self.buf_len as i32;

                    if tail_len > 0 {
                        let head_len = char_len - tail_len as u8;
                        self.set_remain(a, head_len, tail_len as u8);
                        self.set_load_buf();
                    } else {
                        return Some(Ok(self.chr(char_len, a)));
                    }
                } else if self.cursor == self.buf_len {
                    self.set_load_buf();
                } else {
                    unreachable!()
                }
            } else if self.buf_len < self.buf.len() {
                // println!("reach the le!");
                // 如果当前的cur_len小于buf.len()，自然是读到最后一轮了
                // 那么我们再次判断cursor与self.cur_len的关系
                if self.cursor == self.buf_len {
                    return None;
                } else if self.cursor < self.buf_len {
                    let (len, a) = self.cur_u8_unite_len();
                    return Some(Ok(self.chr(len, a)));
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        }
    }
}

// 0xxxxxxx
// 110xxxxx 10xxxxxx
// 1110xxxx 10xxxxxx 10xxxxxx
// 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
#[allow(clippy::unusual_byte_groupings)]
pub const fn utf8_len(first: u8) -> u8 {
    if first < 0b110_00000 {
        1
    } else if first < 0b1110_0000 {
        2
    } else if first < 0b11110_000 {
        3
    } else {
        4
    }
}

#[test]
fn test2() {
    let f = File::open("../123.txt").unwrap();
    let chars = ChrIter::new(f);
    for c in chars {
        println!("{:?}", c);
    }
}
// 0xxxxxxx
// 110xxxxx 10xxxxxx
// 1110xxxx 10xxxxxx 10xxxxxx
// 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
// pub fn unicode(chr: Chre) -> char {
//     unsafe {
//         match chr {
//             Chre::OneByte(bytes) => *bytes.get_unchecked(0) as char,
//             Chre::TwoByte(bytes) => {
// 				let mut chr = 0_u32;
//     			let &a = bytes.get_unchecked(0);
//     			let &b = bytes.get_unchecked(1);
// 				let b_last_six = mask_tail_six(b) as u32;
// 				chr |= b_last_six;
// 				let a_last_five = mask_tail_five(a) as u32;

// 				todo!()
//     		},
//             Chre::ThreeByte(_) => todo!(),
//             Chre::FourByte(_) => todo!(),
//         }
//     }
// }

// const fn mask_tail_six(byte: u8) -> u8 {
//     const MASK: u8 = 0b00_111111;
//     byte & MASK
// }

// const fn mask_tail_five(byte: u8) -> u8 {
//     const MASK: u8 = 0b000_11111;
//     byte & MASK
// }

// const fn mask_tail_four(byte: u8) -> u8 {
//     const MASK: u8 = 0b0000_1111;
//     byte & MASK
// }

// const fn mask_tail_three(byte: u8) -> u8 {
//     const MASK: u8 = 0b00000_111;
//     byte & MASK
// }

// #[test]
// fn test() {
//     println!("0b{:08b}", mask_tail_six(0b10_111101));
// }

// #[test]
// fn test() {
//     assert_eq!(1, utf8_len(b'a'));
//     println!(
//         "{:?}",
//         std::str::from_utf8(&[0b11110_010, 0b10_000000, 0b10_000000, 0b10_000000])
//     );
//     println!("{}", '\u{80000}');
//     println!("{:?}", '򀀀'.to_string().as_bytes());

//     assert_eq!(2, utf8_len('£'.to_string().as_bytes()[0]));
//     assert_eq!(3, utf8_len('我'.to_string().as_bytes()[0]));
//     assert_eq!(4, utf8_len('򀀀'.to_string().as_bytes()[0]));
// }
