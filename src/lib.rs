use std::ptr;
use std::ops;
use std::os::raw::{c_int,c_char,c_uint};
use std::ffi::CStr;

#[repr(C)]
#[derive(PartialEq,Eq,Debug)]
struct LuaMatch {
    start: c_int,
    end: c_int
}

static LUA_MAXCAPTURES: usize = 32;

#[link(name = "lua-str", kind="static")]
extern {
    fn str_match (
        s: *const u8, ls: c_uint, p: *const u8, lp: c_uint,
        err_msg: *mut *mut c_char,
        mm: *mut LuaMatch
        ) -> c_int;
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
        let c_ptr: *mut c_char = ptr::null_mut();
        let pvoid = Box::into_raw(Box::new(c_ptr));
        let err_msg : *mut *mut c_char = pvoid;

        unsafe {
           self.n_match = str_match(s.as_ptr(),s.len() as c_uint,
                self.patt.as_ptr(),self.patt.len() as c_uint,
                err_msg, self.matches.as_mut_ptr()) as usize;
            let ep = *err_msg;
            if ! ep.is_null() {
                panic!(format!("lua-pattern {:?}",CStr::from_ptr(ep)));
            }
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
            vec.push(&text[self.capture(i)]);
        }
        self.n_match > 0
    }

    pub fn range(&self) -> ops::Range<usize> {
        self.capture(0)
    }

    pub fn capture(&self, i: usize) -> ops::Range<usize> {
        ops::Range{
            start: self.matches[i].start as usize,
            end: self.matches[i].end as usize
        }
    }

    pub fn first_match<'b>(&mut self, text: &'b str) -> Option<&'b str> {
        self.matches(text);
        if self.n_match > 0 {
            Some(&text[self.capture(if self.n_match > 1 {1} else {0})])
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
            let all = self.range();
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
        &self.text[self.m.capture(i)]
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
            let slice = &self.text[self.m.capture(first)];
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
        assert_eq!(m.captures(" one two"), &["one two","one"]);
        let empty: &[&str] = &[];
        assert_eq!(m.captures("four"), empty);

        assert_eq!(m.matches("one dog"), true);
        assert_eq!(m.matches("dog one "), true);
        assert_eq!(m.matches("dog one"), false);

        let text = "one dog";
        let mut m = LuaPattern::new("^(%a+)");
        assert_eq!(m.matches(text), true);
        assert_eq!(&text[m.capture(1)], "one");
        assert_eq!(m.matches(" one dog"), false);

        // captures without allocation
        let captures = m.match_captures(text);
        assert_eq!(captures.get(0), "one");
        assert_eq!(captures.get(1), "one");

        let mut m = LuaPattern::new("(%S+)%s*=%s*(.+)");

        //  captures as Vec
        let cc = m.captures(" hello= bonzo dog");
        assert_eq!(cc[0], "hello= bonzo dog");
        assert_eq!(cc[1], "hello");
        assert_eq!(cc[2], "bonzo dog");

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
        use std::collections::HashMap;
        
        let mut m = LuaPattern::new("%$(%S+)");
        let res = m.gsub("hello $dolly you're so $fine!",
            |cc| cc.get(1).to_uppercase()
        );
        assert_eq!(res,"hello DOLLY you're so FINE!");
        
        let mut map = HashMap::new();
        map.insert("dolly", "baby");
        map.insert("fine", "cool");
        map.insert("good-looking", "pretty");
        
        let mut m = LuaPattern::new("%$%((.-)%)");
        let res = m.gsub("hello $(dolly) you're so $(fine) and $(good-looking)",
            |cc| map.get(cc.get(1)).unwrap_or(&"?").to_string()
        );
        assert_eq!(res,"hello baby you're so cool and pretty");
        
    }
}
