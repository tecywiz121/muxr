error_chain! {
    links {
        Muxr(::muxr_core::error::Error, ::muxr_core::error::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Bincode(::bincode::Error);
    }
}
