extern crate lua_patterns;
use lua_patterns::LuaPattern;

fn main() {
    let mut m = LuaPattern::new("(%a+) one");
    let text = " hello one two";
    assert!(m.matches(text));
    assert_eq!(m.capture(1),1..6);
    assert_eq!(m.capture(0),1..10);
}
