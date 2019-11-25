use bincode;

error_chain! {
    links {
        Muxr(::muxr_core::error::Error, ::muxr_core::error::ErrorKind);
    }

    foreign_links {
        Nix(::nix::Error);
        Io(::std::io::Error);
        Bincode(bincode::Error);
    }
}
