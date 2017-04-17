extern crate lua_patterns as lp;



fn main() {

    let mut m = lp::LuaPattern::new("hello%");
    m.matches("hello");
    println!("ok");

    ///*
    let mut m = lp::LuaPattern::new("(%a+)");
    let mut iter = m.gmatch("one two three");
    assert_eq!(iter.next(), Some("one"));
    assert_eq!(iter.next(), Some("two"));
    assert_eq!(iter.next(), Some("three"));
    assert_eq!(iter.next(), None);

    let mut m = lp::LuaPattern::new("(%a+)");
    let split: Vec<_> = m.gmatch("dog cat leopard wolf").collect();
    assert_eq!(split,&["dog","cat","leopard","wolf"]);

    let mut m = lp::LuaPattern::new("(%S+)%s*=%s*(.+)");
    let cc = m.captures(" hello= bonzo dog");
    assert_eq!(cc[0], "hello= bonzo dog");
    assert_eq!(cc[1],"hello");
    assert_eq!(cc[2],"bonzo dog");

    let captures = m.match_captures(" frodo = baggins");
    for s in captures {
        println!("{:?}",s);
    }


    let mut m = lp::LuaPattern::new("%$(%S+)");
    let res = m.gsub("hello $dolly you're so $fine",
        |cc| cc.get(1).to_uppercase()
    );
    assert_eq!(res,"hello DOLLY you're so FINE");
    //*/




}
