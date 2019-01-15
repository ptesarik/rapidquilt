#[cfg(unix)]
fn bytes_to_pathbuf(bytes: &[u8]) -> PathBuf {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    PathBuf::from(OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn bytes_to_pathbuf(bytes: &[u8]) -> PathBuf {
    // XXX: This may not work in case of some really weird paths (control characters
    //      and what not). But I guess those can not be legaly saved in patch files
    //      anyway.

    PathBuf::from(String::from_utf8_lossy(bytes).as_ref())
}


                    Filename::Real(bytes_to_pathbuf(&f[..]))
                    Filename::Real(bytes_to_pathbuf(&f[..]))
        do_parse!(tag!(s!(b"--- ")) >> filename: parse_filename >> take_until_newline_incl >> (MetadataLine::MinusFilename(filename))) |
        do_parse!(tag!(s!(b"+++ ")) >> filename: parse_filename >> take_until_newline_incl >> (MetadataLine::PlusFilename(filename))) |
    // All of them in basic form

    // Filename with date
    assert_parsed!(parse_metadata_line, b"--- a/bla/ble.c	2013-09-23 18:41:09.000000000 -0400\n", MinusFilename(Filename::Real(PathBuf::from("a/bla/ble.c"))));

    pub function: &'a [u8],
/// Parses the line like "@@ -3,4 +5,6 @@ function\n"
        opt!(tag!(c!(b' '))) >>
        function: take_until_newline >>
            function: &function
        function: &b""[..],
        function: &b""[..],
        function: s!(b"function name"),
        header.function
    assert_eq!(h.function, b"place");
    assert_eq!(hs[0].function, b"place1");
    assert_eq!(hs[1].function, b"place2");