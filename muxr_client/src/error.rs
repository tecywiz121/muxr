error_chain! {
    links {
        Muxr(::muxr::error::Error, ::muxr::error::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
    }
}
