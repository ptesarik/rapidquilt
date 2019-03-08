use std::borrow::Cow;
use std::path::{Path, PathBuf};
/// Parses filename as-is included in the patch, delimited by first whitespace. Returns byte slice
/// of the path as-is in the input data.
/// Parses a quoted filename that may contain escaped characters. Returns owned buffer with the
/// unescaped filename.
enum Filename<'a> {
    /// Actual filename, either as byte slice of the patch file or owned buffer.
    Real(Cow<'a, Path>),
    /// The special "/dev/null" filename.
    DevNull,
            // First attempt to parse it as quoted filename. This will reject it quickly if it does
            // not start with '"' character
            map!(parse_filename_quoted, |filename_vec| {
                if &filename_vec[..] == NULL_FILENAME {
                    let pathbuf = match () {
                        #[cfg(unix)]
                        () => {
                            // We have owned buffer, so we must turn it into allocated `PathBuf`, but
                            // no conversion of encoding is necessary on unix systems.

                            use std::ffi::OsString;
                            use std::os::unix::ffi::OsStringExt;
                            PathBuf::from(OsString::from_vec(filename_vec))
                        }

                        #[cfg(not(unix))]
                        () => {
                            // In non-unix systems, we don't know how is `Path` represented, so we can
                            // not just take the byte slice and use it as `Path`. For example on Windows
                            // paths are a form of UTF-16, while the content of patch file has undefined
                            // encoding and we assume UTF-8. So conversion has to happen.

                            PathBuf::from(String::from_utf8_lossy(bytes).owned())
                        }
                    };

                    Filename::Real(Cow::Owned(pathbuf))
            // Then attempt to parse it as direct filename (without quotes, spaces or escapes)
            map!(parse_filename_direct, |filename| {
                if &filename[..] == NULL_FILENAME {
                    let path = match () {
                        #[cfg(unix)]
                        () => {
                            // We have a byte slice, which we can wrap into `Path` and use it without
                            // any heap allocation.

                            use std::ffi::OsStr;
                            use std::os::unix::ffi::OsStrExt;
                            Cow::Borrowed(Path::new(OsStr::from_bytes(filename.0)))
                        }

                        #[cfg(not(unix))]
                        () => {
                            // In non-unix systems, we don't know how is `Path` represented, so we can
                            // not just take the byte slice and use it as `Path`. For example on Windows
                            // paths are a form of UTF-16, while the content of patch file has undefined
                            // encoding and we assume UTF-8. So conversion has to happen.

                            Cow::Owned(PathBuf::from(String::from_utf8_lossy(bytes).owned()))
                        }
                    };

                    Filename::Real(path)
        assert_parsed!(parse_filename, input, Filename::Real(Cow::Owned(PathBuf::from(result))));
enum MetadataLine<'a> {
    GitDiffSeparator(Filename<'a>, Filename<'a>),
    MinusFilename(Filename<'a>),
    PlusFilename(Filename<'a>),
    assert_parsed!(parse_metadata_line, b"diff --git aaa bbb\n", GitDiffSeparator(Filename::Real(Cow::Owned(PathBuf::from("aaa"))), Filename::Real(Cow::Owned(PathBuf::from("bbb")))));
    assert_parsed!(parse_metadata_line, b"--- aaa\n", MinusFilename(Filename::Real(Cow::Owned(PathBuf::from("aaa")))));
    assert_parsed!(parse_metadata_line, b"+++ aaa\n", PlusFilename(Filename::Real(Cow::Owned(PathBuf::from("aaa")))));
    assert_parsed!(parse_metadata_line, b"--- a/bla/ble.c	2013-09-23 18:41:09.000000000 -0400\n", MinusFilename(Filename::Real(Cow::Owned(PathBuf::from("a/bla/ble.c")))));
    Metadata(MetadataLine<'a>),
    assert_parsed!(parse_patch_line, b"diff --git aaa bbb\n", Metadata(GitDiffSeparator(Filename::Real(Cow::Owned(PathBuf::from("aaa"))), Filename::Real(Cow::Owned(PathBuf::from("bbb"))))));
    assert_parsed!(parse_patch_line, b"--- aaa\n", Metadata(MinusFilename(Filename::Real(Cow::Owned(PathBuf::from("aaa"))))));
struct FilePatchMetadata<'a> {
    old_filename: Option<Filename<'a>>,
    new_filename: Option<Filename<'a>>,
enum FilePatchMetadataBuildError<'a> {
    MissingFilenames(FilePatchMetadata<'a>),
impl<'a> FilePatchMetadata<'a> {
    pub fn build_filepatch(self, hunks: HunksVec<'a, &'a [u8]>) -> Result<TextFilePatch<'a>, FilePatchMetadataBuildError<'a>> {
        let builder = FilePatchBuilder::<&[u8]>::default();
        let builder = builder.kind(self.recognize_kind(&hunks));
        let builder = if self.rename_from && self.rename_to {
            builder.is_rename(true)

            builder
        };
        let builder = builder
            .old_filename(old_filename)
            .new_filename(new_filename)
            // Set the permissions
            .old_permissions(self.old_permissions)
            .new_permissions(self.new_permissions)
            // Set the hunks
            .hunks(hunks);
    pub fn build_hunkless_filepatch(self) -> Result<TextFilePatch<'a>, FilePatchMetadataBuildError<'a>> {
    assert_eq!(file_patch.old_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.new_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.new_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.old_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.new_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.old_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.old_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));
    assert_eq!(file_patch.new_filename(), Some(&Cow::Owned(PathBuf::from("filename1"))));