extern crate cc;

fn main() {
    cc::Build::new().file("src/pty.c").compile("pty_helper");
}
