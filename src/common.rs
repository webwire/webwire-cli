use nom::AsBytes;
use nom_locate::LocatedSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct FilePosition {
    pub line: u32,
    pub column: usize,
}

impl<T: AsBytes> From<LocatedSpan<T>> for FilePosition {
    fn from(span: LocatedSpan<T>) -> Self {
        Self {
            line: span.location_line(),
            column: span.get_utf8_column(),
        }
    }
}
