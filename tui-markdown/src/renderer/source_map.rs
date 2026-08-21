//! Source ranges for rendered top-level Markdown blocks.

use std::ops::Range;

use pulldown_cmark::{Event, Parser};
use ratatui_core::text::Text;

use super::{parse_options, TextWriter};
use crate::{Options, StyleSheet};

/// Renders Markdown and returns top-level source and rendered-line ranges.
///
/// Source ranges are byte offsets into the exact `input` passed to this function. Line ranges index
/// [`SourceMappedText::text`]. Separator lines inserted between top-level blocks are not assigned
/// to either neighboring block.
///
/// Rendering and range collection share one `pulldown-cmark` event pass. Use
/// [`crate::from_str_with_options`] when source metadata is not needed.
///
/// # Example
///
/// ```
/// use tui_markdown::{from_str_with_options_and_source_map, Options};
///
/// let options = Options::default();
/// let mapped = from_str_with_options_and_source_map("# Status\n\nReady", &options);
///
/// assert_eq!(mapped.text().to_string(), "# Status\n\nReady");
/// assert_eq!(mapped.blocks().len(), 2);
/// assert_eq!(mapped.blocks()[0].source_range(), 0..9);
/// assert_eq!(mapped.last_top_level_block_start(), Some(10));
/// ```
pub fn from_str_with_options_and_source_map<'a, S>(
    input: &'a str,
    options: &Options<S>,
) -> SourceMappedText<'a>
where
    S: StyleSheet,
{
    let parser = Parser::new_ext(input, parse_options());
    let has_reference_definitions = parser.reference_definitions().iter().next().is_some();
    let events = parser.into_offset_iter();

    // Source mapping owns the offset iterator. The writer still receives the same Event values as
    // the legacy path, so all Markdown rendering remains in TextWriter::handle_event.
    let writer = TextWriter::new(
        std::iter::empty::<Event<'a>>(),
        options.styles.clone(),
        options.image_fallback,
    );
    #[cfg(feature = "highlight-code")]
    let mut writer = writer.with_code_theme(options.selected_code_theme());
    #[cfg(not(feature = "highlight-code"))]
    let mut writer = writer;

    let mut source_map = SourceMapBuilder::default();
    for (event, source_range) in events {
        let starts_tag = matches!(event, Event::Start(_));
        let ends_tag = matches!(event, Event::End(_));
        let standalone = !starts_tag && !ends_tag && source_map.tag_depth == 0;

        if source_map.tag_depth == 0 && (starts_tag || standalone) {
            source_map.current_block = Some((
                source_range.start,
                writer.text.lines.len() + usize::from(writer.needs_newline),
            ));
        }
        if starts_tag {
            source_map.tag_depth += 1;
        }

        writer.handle_event(event);

        if ends_tag {
            source_map.tag_depth = source_map.tag_depth.saturating_sub(1);
        }
        if (ends_tag && source_map.tag_depth == 0) || standalone {
            source_map.finish_block(source_range.end, writer.text.lines.len());
        }
    }

    SourceMappedText {
        text: writer.text,
        blocks: source_map.blocks,
        has_reference_definitions,
    }
}

/// A rendered top-level Markdown block and its corresponding source range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceMappedBlock {
    source_range: Range<usize>,
    line_range: Range<usize>,
}

impl SourceMappedBlock {
    /// Returns the block's byte range in the original Markdown source.
    #[must_use]
    pub fn source_range(&self) -> Range<usize> {
        self.source_range.clone()
    }

    /// Returns the block's line range in the rendered [`Text`].
    #[must_use]
    pub fn line_range(&self) -> Range<usize> {
        self.line_range.clone()
    }
}

/// Canonical rendered text plus top-level source metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceMappedText<'a> {
    text: Text<'a>,
    blocks: Vec<SourceMappedBlock>,
    has_reference_definitions: bool,
}

impl<'a> SourceMappedText<'a> {
    /// Returns the rendered Markdown text.
    #[must_use]
    pub fn text(&self) -> &Text<'a> {
        &self.text
    }

    /// Consumes the projection and returns its rendered Markdown text.
    #[must_use]
    pub fn into_text(self) -> Text<'a> {
        self.text
    }

    /// Returns the rendered top-level blocks in source order.
    #[must_use]
    pub fn blocks(&self) -> &[SourceMappedBlock] {
        &self.blocks
    }

    /// Returns the byte offset of the final rendered top-level block.
    #[must_use]
    pub fn last_top_level_block_start(&self) -> Option<usize> {
        self.blocks.last().map(|block| block.source_range.start)
    }

    /// Returns whether the source contains reference-style link definitions.
    ///
    /// Such definitions can affect links outside their own source position, so incremental
    /// consumers should not assume that preceding blocks remain stable.
    #[must_use]
    pub fn has_reference_definitions(&self) -> bool {
        self.has_reference_definitions
    }
}

#[derive(Default)]
struct SourceMapBuilder {
    blocks: Vec<SourceMappedBlock>,
    current_block: Option<(usize, usize)>,
    tag_depth: usize,
}

impl SourceMapBuilder {
    fn finish_block(&mut self, source_end: usize, rendered_line_count: usize) {
        if let Some((source_start, line_start)) = self.current_block.take() {
            self.blocks.push(SourceMappedBlock {
                source_range: source_start..source_end,
                line_range: line_start.min(rendered_line_count)..rendered_line_count,
            });
        }
    }
}
