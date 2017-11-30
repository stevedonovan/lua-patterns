extern crate lua_patterns;

fn main() {
   let bad = [
    ("bonzo %","malformed pattern (ends with '%')"),
    ("bonzo (dog%(","unfinished capture"),
    ("alles [%a%[","malformed pattern (missing ']')"),
    ("bonzo (dog (cat)","unfinished capture"),
    ("frodo %f[%A","malformed pattern (missing ']')"),
    ("frodo (1) (2(3)%2)%1","invalid capture index %2"),
    ];    
    
    fn error(s: &str) -> lua_patterns::PatternError {
            lua_patterns::PatternError(s.into())
    }
    
    for p in bad.iter() {
        let res = lua_patterns::LuaPattern::new_try(p.0);
        if let Err(e) = res {
            assert_eq!(e, error(p.1));
        } else {
            println!("'{}' was fine",p.0);
        }
   } 
    
}
