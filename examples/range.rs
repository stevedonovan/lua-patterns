extern crate lua_patterns;
use lua_patterns::LuaPattern;

fn main() {
    let mut m = LuaPattern::new("(%a+) one");
    let text = " hello one two";
    assert!(m.matches(text));
    assert_eq!(m.capture(1),1..6);
    assert_eq!(m.capture(0),1..10);

    let v = m.captures(text);
    assert_eq!(v, &["hello one","hello"]);

    let mut v = Vec::new();
    assert!(m.capture_into(text,&mut v));
    assert_eq!(v, &["hello one","hello"]);

    let patt = &[0xDE,0x00,b'+',0xBE];
    let bytes = &[0xFF,0xEE,0x0,0xDE,0x0,0x0,0xBE,0x0,0x0];

    let mut m = LuaPattern::from_bytes(patt);
    assert!(m.matches_bytes(bytes));
    assert_eq!(&bytes[m.capture(0)], &[0xDE,0x00,0x00,0xBE]);
}
