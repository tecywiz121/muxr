error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Bincode(::bincode::Error);
    }
}
