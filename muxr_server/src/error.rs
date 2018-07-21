error_chain! {
    links {
        Muxr(::muxr::error::Error, ::muxr::error::ErrorKind);
    }

    foreign_links {
        Nix(::nix::Error);
        Io(::std::io::Error);
    }
}
