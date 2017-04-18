## Lua string patterns in Rust

[Lua string patterns](https://www.lua.org/pil/20.2.html) are a powerful
yet lightweight alternative to full regular expressions. They are not
regexps, since there is no alternation (the `|` operator), but this
is not usually a problem. In fact, full regexps become _too powerful_ and
power can be dangerous or just plain confusing.
This is why OpenBSD's httpd has [Lua patterns](http://man.openbsd.org/patterns.7).
The decision to use `%` as the escape rather than the traditional `\` is refreshing.
In the Rust context, `lua-patterns` is a very lightweight dependency, if you
don't need the full power of the `regex` crate.

This library reuses the original source from Lua 5.2 - only
400 lines of battle-tested C. I originally did this for a similar project to bring
[these patterns to C++](https::/github.com/stevedonovan/rx-cpp).

More information can be found on [the Lua wiki](http://lua-users.org/wiki/PatternsTutorial).

I've organized the Rust interface much as the original Lua library, 'match',
'gmatch' and 'gsub', but made these methods of a `LuaPattern` struct. This is
for two main reasons:

  - although string patterns are not compiled, they can be validated upfront
  - after a match, the struct contains the results

```rust
extern crate lua_patterns;
use lua_patterns::LuaPattern;

let mut m = LuaPattern::new("one");
let text = "hello one two";
assert!(m.matches(text));
let r = m.range();
assert_eq!(r.start, 6);
assert_eq!(r.end, 9);
```
This not in itself impressive, since it can be done with the string `find`
method, but once we start using patterns it gets more exciting, especially
with _captures_:

```rust
let mut m = LuaPattern::new("(%a+) one");
let text = " hello one two";
assert!(m.matches(text));
assert_eq!(m.capture(0),1..10); // "hello one"
assert_eq!(m.capture(1),1..6); // "hello"
```
Lua patterns (like regexps) are not anchored by default, so this finds
the first match and works from there. The 0 capture always exists
(the full match) and here the 1 capture just picks up the first word.


