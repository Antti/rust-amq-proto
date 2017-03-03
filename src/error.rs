error_chain! {
    errors {
        Protocol(t: String) {
            description("protocol error")
            display("protocol error: '{}'", t)
        }
    }

    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
    }
}
