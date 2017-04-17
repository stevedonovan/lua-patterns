extern crate gcc;

fn main() {
    gcc::compile_library("liblua-str.a", &["src/lua-str.c"]);
}
