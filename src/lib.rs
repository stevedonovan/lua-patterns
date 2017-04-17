extern crate libc;
use libc::{size_t,c_int};
use std::ptr;
use std::ops;

#[repr(C)]
#[derive(PartialEq,Eq,Debug)]
struct LuaMatch {
    start: c_int,
    end: c_int
}

static LUA_MAXCAPTURES: usize = 32;

// int str_match (const char *s, size_t ls, const char *p, size_t lp, char **err_msg, LuaMatch *mm)
#[link(name = "lua-str", kind="static")]
extern {
    fn str_match (s: *const u8, ls: size_t, p: *const u8, lp: size_t,
        //err_msg: *mut *const u8,
        err_msg: *const u8,
        mm: *mut LuaMatch) -> c_int;
}

pub struct LuaPattern<'a> {
    patt: &'a [u8],
    matches: Vec<LuaMatch>,
    n_match: usize
}

impl <'a> LuaPattern<'a> {
    pub fn new(patt: &'a str) -> LuaPattern<'a> {
        LuaPattern::from_bytes(patt.as_bytes())
    }

    pub fn from_bytes (bytes: &'a [u8]) -> LuaPattern<'a> {
        let mut matches: Vec<LuaMatch> = Vec::with_capacity(LUA_MAXCAPTURES);
        unsafe {matches.set_len(LUA_MAXCAPTURES);}
        LuaPattern{patt: bytes, matches: matches, n_match: 0}
    }

    pub fn matches_bytes(&mut self, s: &[u8]) -> bool {
        let err_msg: *const u8 = ptr::null();

        unsafe {
           self.n_match = str_match(s.as_ptr(),s.len() as size_t,
                self.patt.as_ptr(),self.patt.len() as size_t,
                err_msg, self.matches.as_mut_ptr()) as usize;
        }
        self.n_match > 0
    }

    pub fn matches(&mut self, text: &str) -> bool {
        self.matches_bytes(text.as_bytes())
    }


    pub fn captures<'b>(&mut self, text: &'b str) -> Vec<&'b str> {
        let mut res = Vec::new();
        self.capture_into(text, &mut res);
        res
    }

    pub fn match_captures<'b>(&'a mut self, text: &'b str) -> Captures<'a,'b> {
        self.matches(text);
        Captures {m: self, text: text}
    }

    pub fn capture_into<'b>(&mut self, text: &'b str, vec: &mut Vec<&'b str>) -> bool {
        self.matches(text);
        vec.clear();
        for i in 0..self.n_match {
            vec.push(&text[self.bounds(i)]);
        }
        self.n_match > 0
    }

    pub fn range(&self) -> ops::Range<usize> {
        self.bounds(0)
    }

    pub fn bounds(&self, i: usize) -> ops::Range<usize> {
        ops::Range{
            start: self.matches[i].start as usize,
            end: self.matches[i].end as usize
        }
    }

    pub fn first_match<'b>(&mut self, text: &'b str) -> Option<&'b str> {
        self.matches(text);
        if self.n_match > 0 {
            Some(&text[self.bounds(if self.n_match > 1 {1} else {0})])
        } else {
            None
        }
    }

    pub fn gmatch<'b>(&'a mut self, text: &'b str) -> GMatch<'a,'b> {
        GMatch{m: self, text: text}
    }

    pub fn gsub <F> (&mut self, text: &str, lookup: F) -> String
    where F: Fn(Captures)-> String {
        let mut slice = text;
        let mut res = String::new();
        while self.matches(slice) {
            // full range of match
            let all = self.bounds(0);
            let captures = Captures{m: self, text: slice};
            let repl = lookup(captures);
            // append everything up to match
            res.push_str(&slice[0..all.start]);
            res.push_str(&repl);
            slice = &slice[all.end..];
        }
        res.push_str(slice);
        res
    }

}

pub struct Captures<'a,'b> {
    m: &'a LuaPattern<'a>,
    text: &'b str
}

impl <'a,'b> Captures<'a,'b> {
    pub fn get(&self, i: usize) -> &'b str {
        &self.text[self.m.bounds(i)]
    }

    pub fn num_matches(&self) -> usize {
        self.m.n_match
    }
}

pub struct CaptureIter<'a,'b> {
    cc: Captures<'a,'b>,
    idx: usize,
    top: usize
}

impl <'a,'b>Iterator for CaptureIter<'a,'b> {
    type Item = &'b str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.top {
            let res = self.cc.get(self.idx);
            self.idx += 1;
            Some(res)
        } else {
            None
        }
    }
}

impl <'a,'b> IntoIterator for Captures<'a,'b> {
    type Item = &'b str;
    type IntoIter = CaptureIter<'a,'b>;

    fn into_iter(self) -> Self::IntoIter {
        CaptureIter{idx: 0, top: self.num_matches(),cc: self}
    }
}


pub struct GMatch<'a,'b> {
    m: &'a mut LuaPattern<'a>,
    text: &'b str
}

impl <'a,'b>Iterator for GMatch<'a,'b> {
    type Item = &'b str;

    fn next(&mut self) -> Option<Self::Item> {
        if ! self.m.matches(self.text) {
            None
        } else {
            let first = if self.m.n_match > 1 {1} else {0};
            let slice = &self.text[self.m.bounds(first)];
            self.text = &self.text[self.m.range().end..];
            Some(slice)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn captures_and_matching() {
        let mut m = LuaPattern::new("(one).+");
        assert_eq!(m.captures(" one two"),&["one two","one"]);
        let empty: &[&str] = &[];
        assert_eq!(m.captures("four"),empty);

        assert_eq!(m.matches("one dog"),true);
        assert_eq!(m.matches("dog one "),true);
        assert_eq!(m.matches("dog one"),false);

        let text = "one dog";
        let mut m = LuaPattern::new("^(%a+)");
        assert_eq!(m.matches(text),true);
        assert_eq!(&text[m.bounds(1)], "one");
        assert_eq!(m.matches(" one dog"),false);

        // captures without allocation
        let captures = m.match_captures(text);
        assert_eq!(captures.get(0),"one");
        assert_eq!(captures.get(1),"one");

        let mut m = LuaPattern::new("(%S+)%s*=%s*(.+)");

        //  captures as Vec
        let cc = m.captures(" hello= bonzo dog");
        assert_eq!(cc[0], "hello= bonzo dog");
        assert_eq!(cc[1],"hello");
        assert_eq!(cc[2],"bonzo dog");

        // captures as iterator
        let mut iter = m.match_captures(" frodo = baggins").into_iter();
        assert_eq!(iter.next(), Some("frodo = baggins"));
        assert_eq!(iter.next(), Some("frodo"));
        assert_eq!(iter.next(), Some("baggins"));
        assert_eq!(iter.next(), None);



    }

    #[test]
    fn gmatch() {
        let mut m = LuaPattern::new("%a+");
        let mut iter = m.gmatch("one two three");
        assert_eq!(iter.next(), Some("one"));
        assert_eq!(iter.next(), Some("two"));
        assert_eq!(iter.next(), Some("three"));
        assert_eq!(iter.next(), None);

        let mut m = LuaPattern::new("(%a+)");
        let mut iter = m.gmatch("one two three");
        assert_eq!(iter.next(), Some("one"));
        assert_eq!(iter.next(), Some("two"));
        assert_eq!(iter.next(), Some("three"));
        assert_eq!(iter.next(), None);

    }

    #[test]
    fn gsub() {
        let mut m = LuaPattern::new("%$(%S+)");
        let res = m.gsub("hello $dolly you're so $fine!",
            |cc| cc.get(1).to_uppercase()
        );
        assert_eq!(res,"hello DOLLY you're so FINE!");


    }
}
