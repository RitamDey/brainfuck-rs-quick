use super::tty_read::ReaderOptions;



/// An options object, that defines how the brainfuck interpreter is used.
pub struct Options {
    /// Buffer output until the program finishes executing.
    pub buffer: bool,

    /// Terminal reader options.
    pub reader_options: ReaderOptions,

    /// Profile steps in this interpreter.
    pub profile: bool,
}

impl Options {
    /// Create a default options object.
    pub fn default(buffer: bool, profile: bool) -> Options {
        Options {
            buffer,
            reader_options: ReaderOptions::default(),
            profile,
        }
    }
}
