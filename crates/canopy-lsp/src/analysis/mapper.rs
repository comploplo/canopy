//! Position mapping between LSP and Canopy
//!
//! Converts between LSP positions (line, character) and byte offsets.

use tower_lsp::lsp_types::{Position, Range};

/// Maps between LSP positions and byte offsets.
#[derive(Debug, Clone)]
pub struct PositionMapper {
    /// Byte offset of each line start.
    line_starts: Vec<usize>,
    /// Total byte length.
    byte_length: usize,
}

impl PositionMapper {
    /// Create a new position mapper for the given content.
    #[must_use]
    pub fn new(content: &str) -> Self {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(content.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        Self {
            line_starts,
            byte_length: content.len(),
        }
    }

    /// Get the number of lines in the document.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Get the byte length of the document.
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Convert byte offset to LSP Position (line, character).
    ///
    /// Returns `None` if the byte offset is out of bounds.
    #[must_use]
    pub fn byte_to_position(&self, byte_offset: usize) -> Option<Position> {
        if byte_offset > self.byte_length {
            return None;
        }

        // Binary search for the line containing this offset
        let line = self
            .line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);

        let character = byte_offset.saturating_sub(self.line_starts[line]);

        Some(Position::new(
            u32::try_from(line).unwrap_or(u32::MAX),
            u32::try_from(character).unwrap_or(u32::MAX),
        ))
    }

    /// Convert LSP Position to byte offset.
    ///
    /// Returns `None` if the position is out of bounds.
    #[must_use]
    pub fn position_to_byte(&self, pos: Position) -> Option<usize> {
        let line = pos.line as usize;
        if line >= self.line_starts.len() {
            return None;
        }

        let byte_offset = self.line_starts[line] + pos.character as usize;
        if byte_offset > self.byte_length {
            return None;
        }

        Some(byte_offset)
    }

    /// Convert a byte span (start, end) to an LSP Range.
    ///
    /// Returns `None` if either bound is out of range.
    #[must_use]
    pub fn byte_span_to_range(&self, start: usize, end: usize) -> Option<Range> {
        let start_pos = self.byte_to_position(start)?;
        let end_pos = self.byte_to_position(end)?;
        Some(Range::new(start_pos, end_pos))
    }

    /// Get the line number for a byte offset.
    #[must_use]
    pub fn byte_to_line(&self, byte_offset: usize) -> Option<u32> {
        if byte_offset > self.byte_length {
            return None;
        }

        let line = self
            .line_starts
            .partition_point(|&start| start <= byte_offset)
            .saturating_sub(1);

        u32::try_from(line).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line() {
        let content = "Hello world.";
        let mapper = PositionMapper::new(content);

        assert_eq!(mapper.line_count(), 1);
        assert_eq!(mapper.byte_length(), 12);

        assert_eq!(mapper.byte_to_position(0), Some(Position::new(0, 0)));
        assert_eq!(mapper.byte_to_position(6), Some(Position::new(0, 6)));
        assert_eq!(mapper.byte_to_position(12), Some(Position::new(0, 12)));
    }

    #[test]
    fn test_multi_line() {
        let content = "Hello world.\nSecond line.\nThird.";
        let mapper = PositionMapper::new(content);

        assert_eq!(mapper.line_count(), 3);

        // First line
        assert_eq!(mapper.byte_to_position(0), Some(Position::new(0, 0)));
        assert_eq!(mapper.byte_to_position(5), Some(Position::new(0, 5)));

        // Second line starts at byte 13 (after "Hello world.\n")
        assert_eq!(mapper.byte_to_position(13), Some(Position::new(1, 0)));
        assert_eq!(mapper.byte_to_position(19), Some(Position::new(1, 6)));

        // Third line
        assert_eq!(mapper.byte_to_position(26), Some(Position::new(2, 0)));
    }

    #[test]
    fn test_position_to_byte() {
        let content = "Hello world.\nSecond line.";
        let mapper = PositionMapper::new(content);

        assert_eq!(mapper.position_to_byte(Position::new(0, 0)), Some(0));
        assert_eq!(mapper.position_to_byte(Position::new(0, 6)), Some(6));
        assert_eq!(mapper.position_to_byte(Position::new(1, 0)), Some(13));
        assert_eq!(mapper.position_to_byte(Position::new(1, 6)), Some(19));
    }

    #[test]
    fn test_byte_span_to_range() {
        let content = "John gave Mary a book.";
        let mapper = PositionMapper::new(content);

        // "gave" is at bytes 5-9
        let range = mapper.byte_span_to_range(5, 9);
        assert_eq!(
            range,
            Some(Range::new(Position::new(0, 5), Position::new(0, 9)))
        );
    }

    #[test]
    fn test_out_of_bounds() {
        let content = "Hello";
        let mapper = PositionMapper::new(content);

        assert_eq!(mapper.byte_to_position(100), None);
        assert_eq!(mapper.position_to_byte(Position::new(5, 0)), None);
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let mapper = PositionMapper::new(content);

        assert_eq!(mapper.line_count(), 1);
        assert_eq!(mapper.byte_length(), 0);
        assert_eq!(mapper.byte_to_position(0), Some(Position::new(0, 0)));
    }
}
