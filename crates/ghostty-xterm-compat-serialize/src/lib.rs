use libghostty_vt::{
    error::Error as GhosttyError,
    ffi,
    fmt::{Format, Formatter, FormatterExtra, FormatterOptions},
    render::{CellIterator, RenderState, RowIterator},
    screen::{Cell, CellContentTag, CellWide},
    style::{PaletteIndex, RgbColor, Style, StyleColor, Underline},
    terminal::{Mode, Point, PointCoordinate},
    Terminal, TerminalOptions,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct FixtureChunk {
    #[serde(rename = "delayMs")]
    #[allow(dead_code)]
    delay_ms: u64,
    #[allow(dead_code)]
    data: String,
}

#[derive(Debug, Deserialize)]
struct FixtureFile {
    name: String,
    #[allow(dead_code)]
    chunks: Vec<FixtureChunk>,
}

#[derive(Debug, Serialize)]
pub struct SerializeOutput {
    #[serde(rename = "fixture", skip_serializing_if = "Option::is_none")]
    pub fixture_name: Option<String>,
    #[serde(rename = "serializedCandidate")]
    pub serialized_candidate: String,
    #[serde(rename = "cursorX")]
    pub cursor_x: u16,
    #[serde(rename = "cursorY")]
    pub cursor_y: u16,
}

#[derive(Clone, Debug)]
struct SnapshotCell {
    chars: String,
    width: u16,
    style: Style,
}

#[derive(Clone, Debug)]
struct SnapshotRow {
    is_wrap_continuation: bool,
    cells: Vec<SnapshotCell>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CursorState {
    row: usize,
    col: usize,
}

#[derive(Clone, Copy, Debug)]
struct ScreenCursorMeta {
    scrollback_rows: usize,
    cursor_x: u16,
    cursor_y: u16,
    cursor_style: Style,
}

struct StringSerializeHandler {
    cols: usize,
    rows: usize,
    total_rows: usize,
    palette: [RgbColor; 256],
    all_rows: Vec<String>,
    all_row_separators: Vec<String>,
    current_row: String,
    null_cell_count: usize,
    cursor_style: Style,
    background_style: Style,
    first_row: usize,
    last_cursor: CursorState,
    last_content_cursor: CursorState,
}

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/xterm-compat/fixtures")
        .join(format!("{name}.json"))
}

fn chars_for_grid_ref(
    grid_ref: &libghostty_vt::screen::GridRef<'_>,
) -> Result<String, GhosttyError> {
    let mut buffer = vec!['\0'; 8];

    loop {
        match grid_ref.graphemes(&mut buffer) {
            Ok(len) => return Ok(buffer[..len].iter().collect()),
            Err(GhosttyError::OutOfSpace { required }) => {
                buffer.resize(required.max(buffer.len() * 2), '\0');
            }
            Err(err) => return Err(err),
        }
    }
}

fn style_for_cell(
    grid_ref: &libghostty_vt::screen::GridRef<'_>,
    cell: Cell,
) -> Result<Style, GhosttyError> {
    Ok(match cell.content_tag()? {
        CellContentTag::Codepoint | CellContentTag::CodepointGrapheme => grid_ref.style()?,
        CellContentTag::BgColorPalette => Style {
            bg_color: StyleColor::Palette(cell.bg_color_palette()?),
            ..Style::default()
        },
        CellContentTag::BgColorRgb => Style {
            bg_color: StyleColor::Rgb(cell.bg_color_rgb()?),
            ..Style::default()
        },
    })
}

fn snapshot_rows(
    terminal: &Terminal<'_, '_>,
    screen: ffi::TerminalScreen::Type,
) -> Result<Vec<SnapshotRow>, GhosttyError> {
    let total_rows = match screen {
        ffi::TerminalScreen::PRIMARY => terminal.screen_total_rows(screen)?,
        ffi::TerminalScreen::ALTERNATE => terminal.screen_total_rows(screen)?,
        _ => None,
    }
    .ok_or(GhosttyError::InvalidValue)?;
    let cols = usize::from(terminal.cols()?);
    if screen == terminal.active_screen()? && total_rows == usize::from(terminal.rows()?) {
        let mut render_state = RenderState::new()?;
        let snapshot = render_state.update(terminal)?;
        let mut row_iterator = RowIterator::new()?;
        let mut cell_iterator = CellIterator::new()?;
        let mut rows = Vec::with_capacity(total_rows);

        let mut row_iteration = row_iterator.update(&snapshot)?;
        while let Some(row) = row_iteration.next() {
            let raw_row = row.raw_row()?;
            let is_wrap_continuation = raw_row.is_wrap_continuation()?;
            let mut cells = Vec::with_capacity(cols);
            let mut cell_iteration = cell_iterator.update(row)?;

            for x in 0..cols {
                cell_iteration.select(x as u16)?;
                let raw_cell = cell_iteration.raw_cell()?;
                let width = match raw_cell.wide()? {
                    CellWide::Narrow => 1,
                    CellWide::Wide => 2,
                    CellWide::SpacerTail | CellWide::SpacerHead => 0,
                };
                let chars = if width == 0 {
                    String::new()
                } else {
                    cell_iteration.graphemes()?.into_iter().collect()
                };
                let mut style = cell_iteration.style()?;
                if let Some(fg) = cell_iteration.fg_color()? {
                    style.fg_color = StyleColor::Rgb(fg);
                }
                if let Some(bg) = cell_iteration.bg_color()? {
                    style.bg_color = StyleColor::Rgb(bg);
                }
                cells.push(SnapshotCell {
                    chars,
                    width,
                    style,
                });
            }

            rows.push(SnapshotRow {
                is_wrap_continuation,
                cells,
            });
        }

        return Ok(rows);
    }

    let mut rows = Vec::with_capacity(total_rows);

    for y in 0..total_rows {
        let row_ref = terminal
            .screen_grid_ref(screen, Point::Screen(PointCoordinate { x: 0, y: y as u32 }))?
            .ok_or(GhosttyError::InvalidValue)?;
        let row = row_ref.row()?;
        let is_wrap_continuation = row.is_wrap_continuation()?;
        let mut cells = Vec::with_capacity(cols);

        for x in 0..cols {
            let grid_ref = terminal
                .screen_grid_ref(
                    screen,
                    Point::Screen(PointCoordinate {
                        x: x as u16,
                        y: y as u32,
                    }),
                )?
                .ok_or(GhosttyError::InvalidValue)?;
            let cell = grid_ref.cell()?;
            let width = match cell.wide()? {
                CellWide::Narrow => 1,
                CellWide::Wide => 2,
                CellWide::SpacerTail | CellWide::SpacerHead => 0,
            };
            let chars = if width == 0 {
                String::new()
            } else {
                chars_for_grid_ref(&grid_ref)?
            };
            let style = style_for_cell(&grid_ref, cell)?;
            cells.push(SnapshotCell {
                chars,
                width,
                style,
            });
        }

        rows.push(SnapshotRow {
            is_wrap_continuation,
            cells,
        });
    }

    Ok(rows)
}

fn canonical_style_color(color: StyleColor, palette: &[RgbColor; 256]) -> StyleColor {
    match color {
        StyleColor::Rgb(rgb) => match palette_match_index(palette, rgb) {
            Some(index) => StyleColor::Palette(PaletteIndex(index)),
            None => StyleColor::Rgb(rgb),
        },
        other => other,
    }
}

fn equal_fg(left: Style, right: Style, palette: &[RgbColor; 256]) -> bool {
    canonical_style_color(left.fg_color, palette) == canonical_style_color(right.fg_color, palette)
}

fn equal_bg(left: Style, right: Style, palette: &[RgbColor; 256]) -> bool {
    canonical_style_color(left.bg_color, palette) == canonical_style_color(right.bg_color, palette)
}

fn equal_underline(left: Style, right: Style) -> bool {
    if left.underline == Underline::None && right.underline == Underline::None {
        return true;
    }
    left.underline == right.underline && left.underline_color == right.underline_color
}

fn equal_flags(left: Style, right: Style) -> bool {
    left.inverse == right.inverse
        && left.bold == right.bold
        && left.underline == right.underline
        && equal_underline(left, right)
        && left.overline == right.overline
        && left.blink == right.blink
        && left.invisible == right.invisible
        && left.italic == right.italic
        && left.faint == right.faint
        && left.strikethrough == right.strikethrough
}

fn styles_equal(left: Style, right: Style, palette: &[RgbColor; 256]) -> bool {
    equal_fg(left, right, palette) && equal_bg(left, right, palette) && equal_flags(left, right)
}

fn palette_match_index(palette: &[RgbColor; 256], color: RgbColor) -> Option<u8> {
    palette
        .iter()
        .position(|candidate| *candidate == color)
        .and_then(|index| u8::try_from(index).ok())
}

fn push_color_sgr(
    parts: &mut Vec<String>,
    base_palette: u16,
    truecolor_prefix: u16,
    color: StyleColor,
    palette: &[RgbColor; 256],
) {
    match color {
        StyleColor::None => {
            let reset = if base_palette == 30 { 39 } else { 49 };
            parts.push(reset.to_string());
        }
        StyleColor::Palette(index) => {
            let color = index.0;
            if color >= 16 {
                let prefix = if base_palette == 30 { 38 } else { 48 };
                parts.push(prefix.to_string());
                parts.push("5".to_string());
                parts.push(color.to_string());
            } else {
                let code = if color & 8 != 0 {
                    base_palette + 60 + u16::from(color & 7)
                } else {
                    base_palette + u16::from(color & 7)
                };
                parts.push(code.to_string());
            }
        }
        StyleColor::Rgb(rgb) => {
            if let Some(index) = palette_match_index(palette, rgb) {
                let color = index;
                if color >= 16 {
                    let prefix = if base_palette == 30 { 38 } else { 48 };
                    parts.push(prefix.to_string());
                    parts.push("5".to_string());
                    parts.push(color.to_string());
                } else {
                    let code = if color & 8 != 0 {
                        base_palette + 60 + u16::from(color & 7)
                    } else {
                        base_palette + u16::from(color & 7)
                    };
                    parts.push(code.to_string());
                }
            } else {
                let RgbColor { r, g, b } = rgb;
                parts.push(truecolor_prefix.to_string());
                parts.push("2".to_string());
                parts.push(r.to_string());
                parts.push(g.to_string());
                parts.push(b.to_string());
            }
        }
    }
}

fn underline_style_code(underline: Underline) -> Option<&'static str> {
    match underline {
        Underline::None => None,
        Underline::Single => Some("4"),
        Underline::Double => Some("4:2"),
        Underline::Curly => Some("4:3"),
        Underline::Dotted => Some("4:4"),
        Underline::Dashed => Some("4:5"),
        _ => None,
    }
}

fn diff_style(current: Style, previous: Style, palette: &[RgbColor; 256]) -> Vec<String> {
    if styles_equal(current, previous, palette) {
        return Vec::new();
    }

    let mut sgr = Vec::new();
    let fg_changed = !equal_fg(current, previous, palette);
    let bg_changed = !equal_bg(current, previous, palette);
    let flags_changed = !equal_flags(current, previous);

    if current.is_default() {
        if !previous.is_default() {
            sgr.push("0".to_string());
        }
        return sgr;
    }

    if fg_changed {
        push_color_sgr(&mut sgr, 30, 38, current.fg_color, palette);
    }
    if bg_changed {
        push_color_sgr(&mut sgr, 40, 48, current.bg_color, palette);
    }
    if flags_changed {
        if current.inverse != previous.inverse {
            sgr.push(if current.inverse { "7" } else { "27" }.to_string());
        }
        if current.bold != previous.bold {
            sgr.push(if current.bold { "1" } else { "22" }.to_string());
        }
        if !equal_underline(current, previous) {
            match underline_style_code(current.underline) {
                Some(style) => {
                    sgr.push(style.to_string());
                    match current.underline_color {
                        StyleColor::None => {}
                        StyleColor::Palette(index) => {
                            sgr.push(format!("58:5:{}", index.0));
                        }
                        StyleColor::Rgb(RgbColor { r, g, b }) => {
                            sgr.push(format!("58:2::{r}:{g}:{b}"));
                        }
                    }
                }
                None => sgr.push("24".to_string()),
            }
        } else if current.underline != previous.underline {
            sgr.push(
                if current.underline == Underline::None {
                    "24"
                } else {
                    "4"
                }
                .to_string(),
            );
        }
        if current.overline != previous.overline {
            sgr.push(if current.overline { "53" } else { "55" }.to_string());
        }
        if current.blink != previous.blink {
            sgr.push(if current.blink { "5" } else { "25" }.to_string());
        }
        if current.invisible != previous.invisible {
            sgr.push(if current.invisible { "8" } else { "28" }.to_string());
        }
        if current.italic != previous.italic {
            sgr.push(if current.italic { "3" } else { "23" }.to_string());
        }
        if current.faint != previous.faint {
            sgr.push(if current.faint { "2" } else { "22" }.to_string());
        }
        if current.strikethrough != previous.strikethrough {
            sgr.push(if current.strikethrough { "9" } else { "29" }.to_string());
        }
    }

    sgr
}

impl StringSerializeHandler {
    fn new(cols: usize, rows: usize, total_rows: usize, palette: [RgbColor; 256]) -> Self {
        Self {
            cols,
            rows,
            total_rows,
            palette,
            all_rows: Vec::new(),
            all_row_separators: Vec::new(),
            current_row: String::new(),
            null_cell_count: 0,
            cursor_style: Style::default(),
            background_style: Style::default(),
            first_row: 0,
            last_cursor: CursorState::default(),
            last_content_cursor: CursorState::default(),
        }
    }

    fn serialize(
        mut self,
        snapshot_rows: &[SnapshotRow],
        cursor: ScreenCursorMeta,
    ) -> Result<String, GhosttyError> {
        self.all_rows = vec![String::new(); snapshot_rows.len()];
        self.all_row_separators = vec![String::new(); snapshot_rows.len()];

        for (row_index, row) in snapshot_rows.iter().enumerate() {
            for (col_index, cell) in row.cells.iter().enumerate() {
                self.next_cell(cell, row_index, col_index);
            }
            self.row_end(
                snapshot_rows,
                row_index,
                row_index + 1 == snapshot_rows.len(),
            );
        }

        let mut row_end = self.all_rows.len();
        if self.total_rows <= self.rows {
            row_end = self
                .last_content_cursor
                .row
                .saturating_add(1)
                .saturating_sub(self.first_row);
            self.last_cursor = self.last_content_cursor;
        }

        let mut content = String::new();
        for index in 0..row_end {
            content.push_str(&self.all_rows[index]);
            if index + 1 < row_end {
                content.push_str(&self.all_row_separators[index]);
            }
        }

        let real_cursor_row = cursor.scrollback_rows + usize::from(cursor.cursor_y);
        let real_cursor_col = usize::from(cursor.cursor_x);
        if real_cursor_row != self.last_cursor.row || real_cursor_col != self.last_cursor.col {
            let row_delta = real_cursor_row as isize - self.last_cursor.row as isize;
            if row_delta > 0 {
                content.push_str(&format!("\x1b[{}B", row_delta));
            } else if row_delta < 0 {
                content.push_str(&format!("\x1b[{}A", -row_delta));
            }

            let col_delta = real_cursor_col as isize - self.last_cursor.col as isize;
            if col_delta > 0 {
                content.push_str(&format!("\x1b[{}C", col_delta));
            } else if col_delta < 0 {
                content.push_str(&format!("\x1b[{}D", -col_delta));
            }
        }

        let final_sgr = diff_style(cursor.cursor_style, self.cursor_style, &self.palette);
        if !final_sgr.is_empty() {
            content.push_str(&format!("\x1b[{}m", final_sgr.join(";")));
        }

        Ok(content)
    }

    fn next_cell(&mut self, cell: &SnapshotCell, row: usize, col: usize) {
        if cell.width == 0 {
            return;
        }

        let is_empty_cell = cell.chars.is_empty();
        let sgr_seq = diff_style(cell.style, self.cursor_style, &self.palette);
        let style_changed = if is_empty_cell {
            !equal_bg(self.cursor_style, cell.style, &self.palette)
        } else {
            !sgr_seq.is_empty()
        };

        if style_changed {
            if self.null_cell_count > 0 {
                if !equal_bg(self.cursor_style, self.background_style, &self.palette) {
                    self.current_row
                        .push_str(&format!("\x1b[{}X", self.null_cell_count));
                }
                self.current_row
                    .push_str(&format!("\x1b[{}C", self.null_cell_count));
                self.null_cell_count = 0;
            }

            self.last_cursor = CursorState { row, col };
            self.last_content_cursor = self.last_cursor;
            self.current_row
                .push_str(&format!("\x1b[{}m", sgr_seq.join(";")));
            self.cursor_style = cell.style;
        }

        if is_empty_cell {
            self.null_cell_count += usize::from(cell.width);
        } else {
            if self.null_cell_count > 0 {
                if equal_bg(self.cursor_style, self.background_style, &self.palette) {
                    self.current_row
                        .push_str(&format!("\x1b[{}C", self.null_cell_count));
                } else {
                    self.current_row
                        .push_str(&format!("\x1b[{}X", self.null_cell_count));
                    self.current_row
                        .push_str(&format!("\x1b[{}C", self.null_cell_count));
                }
                self.null_cell_count = 0;
            }

            self.current_row.push_str(&cell.chars);
            self.last_cursor = CursorState {
                row,
                col: col + usize::from(cell.width),
            };
            self.last_content_cursor = self.last_cursor;
        }
    }

    fn row_end(&mut self, rows: &[SnapshotRow], row_index: usize, is_last_row: bool) {
        if self.null_cell_count > 0
            && !equal_bg(self.cursor_style, self.background_style, &self.palette)
        {
            self.current_row
                .push_str(&format!("\x1b[{}X", self.null_cell_count));
        }

        let mut row_separator = String::new();
        if !is_last_row {
            let current_row = &rows[row_index];
            let next_row = &rows[row_index + 1];

            if !next_row.is_wrap_continuation {
                row_separator.push_str("\r\n");
                self.last_cursor = CursorState {
                    row: row_index + 1,
                    col: 0,
                };
            } else {
                let this_row_last_char = &current_row.cells[self.cols - 1];
                let this_row_last_second_char = &current_row.cells[self.cols - 2];
                let next_row_first_char = &next_row.cells[0];
                let is_next_row_first_double_width = next_row_first_char.width > 1;

                let mut is_valid = false;
                if if is_next_row_first_double_width {
                    self.null_cell_count <= 1
                } else {
                    self.null_cell_count == 0
                } {
                    if (!this_row_last_char.chars.is_empty() || this_row_last_char.width == 0)
                        && equal_bg(
                            this_row_last_char.style,
                            next_row_first_char.style,
                            &self.palette,
                        )
                    {
                        is_valid = true;
                    }

                    if is_next_row_first_double_width
                        && (!this_row_last_second_char.chars.is_empty()
                            || this_row_last_second_char.width == 0)
                        && equal_bg(
                            this_row_last_char.style,
                            next_row_first_char.style,
                            &self.palette,
                        )
                        && equal_bg(
                            this_row_last_second_char.style,
                            next_row_first_char.style,
                            &self.palette,
                        )
                    {
                        is_valid = true;
                    }
                }

                if !is_valid {
                    row_separator.push_str(&"-".repeat(self.null_cell_count + 1));
                    row_separator.push_str("\x1b[1D\x1b[1X");

                    if self.null_cell_count > 0 {
                        row_separator.push_str("\x1b[A");
                        row_separator
                            .push_str(&format!("\x1b[{}C", self.cols - self.null_cell_count));
                        row_separator.push_str(&format!("\x1b[{}X", self.null_cell_count));
                        row_separator
                            .push_str(&format!("\x1b[{}D", self.cols - self.null_cell_count));
                        row_separator.push_str("\x1b[B");
                    }

                    self.last_content_cursor = CursorState {
                        row: row_index + 1,
                        col: 0,
                    };
                    self.last_cursor = self.last_content_cursor;
                }
            }
        }

        self.all_rows[row_index] = std::mem::take(&mut self.current_row);
        self.all_row_separators[row_index] = row_separator;
        self.null_cell_count = 0;
    }
}

fn screen_cursor_meta(
    terminal: &Terminal<'_, '_>,
    screen: ffi::TerminalScreen::Type,
) -> Result<Option<ScreenCursorMeta>, GhosttyError> {
    let scrollback_rows = terminal.screen_scrollback_rows(screen)?;
    let cursor_x = terminal.screen_cursor_x(screen)?;
    let cursor_y = terminal.screen_cursor_y(screen)?;
    let cursor_style = terminal.screen_cursor_style(screen)?;

    match (scrollback_rows, cursor_x, cursor_y, cursor_style) {
        (Some(scrollback_rows), Some(cursor_x), Some(cursor_y), Some(cursor_style)) => {
            Ok(Some(ScreenCursorMeta {
                scrollback_rows,
                cursor_x,
                cursor_y,
                cursor_style,
            }))
        }
        (None, None, None, None) => Ok(None),
        _ => Err(GhosttyError::InvalidValue),
    }
}

fn serialize_screen(
    terminal: &Terminal<'_, '_>,
    screen: ffi::TerminalScreen::Type,
) -> Result<Option<String>, GhosttyError> {
    let total_rows = match terminal.screen_total_rows(screen)? {
        Some(total_rows) => total_rows,
        None => return Ok(None),
    };
    let cursor = screen_cursor_meta(terminal, screen)?.ok_or(GhosttyError::InvalidValue)?;
    let rows = snapshot_rows(terminal, screen)?;
    Ok(Some(
        StringSerializeHandler::new(
            usize::from(terminal.cols()?),
            usize::from(terminal.rows()?),
            total_rows,
            terminal.color_palette()?,
        )
        .serialize(&rows, cursor)?,
    ))
}

fn trailing_scrolling_region_suffix(vt: &str) -> &str {
    fn is_region_sequence(sequence: &str) -> bool {
        let Some(body) = sequence.strip_prefix("\x1b[") else {
            return false;
        };
        let Some(final_byte) = body.chars().last() else {
            return false;
        };
        if final_byte != 'r' && final_byte != 's' {
            return false;
        }
        body[..body.len() - final_byte.len_utf8()]
            .chars()
            .all(|ch| ch.is_ascii_digit() || ch == ';')
    }

    let mut start = vt.len();
    while let Some(seq_start) = vt[..start].rfind("\x1b[") {
        let candidate = &vt[seq_start..start];
        if !is_region_sequence(candidate) {
            break;
        }
        start = seq_start;
    }
    &vt[start..]
}

fn scrolling_region_suffix(
    terminal: &Terminal<'_, '_>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut formatter = Formatter::new(
        terminal,
        FormatterOptions {
            format: Format::Vt,
            trim: true,
            unwrap: true,
            extra: FormatterExtra {
                scrolling_region: true,
                ..FormatterExtra::default()
            },
        },
    )?;
    let len = formatter.format_len()?;
    let mut formatted = vec![0u8; len];
    let written = formatter.format_buf(&mut formatted)?;
    let vt = std::str::from_utf8(&formatted[..written])?;
    Ok(trailing_scrolling_region_suffix(vt).to_string())
}

fn split_leading_sgr_prefix(text: &str) -> (&str, &str) {
    let mut end = 0;
    let bytes = text.as_bytes();

    while end + 2 < bytes.len() && bytes[end] == 0x1b && bytes[end + 1] == b'[' {
        let rest = &text[end + 2..];
        let Some(m_index) = rest.find('m') else {
            break;
        };
        let candidate = &rest[..m_index];
        if candidate
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b';')
        {
            end += 2 + m_index + 1;
            continue;
        }
        break;
    }

    text.split_at(end)
}

fn style_prefix(style: Style, palette: &[RgbColor; 256]) -> Option<String> {
    let sgr = diff_style(style, Style::default(), palette);
    if sgr.is_empty() {
        None
    } else {
        Some(format!("\x1b[{}m", sgr.join(";")))
    }
}

fn normalize_xterm_truecolor_token(token: &str) -> Option<String> {
    let mut parts = token.split(':');
    let base = parts.next()?;
    let mode = parts.next()?;
    let first = parts.next()?;
    let second = parts.next()?;
    let third = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    if !matches!(base, "38" | "48" | "58") || mode != "2" || first.is_empty() {
        return None;
    }

    Some(format!("{base};2;{second};{third};0"))
}

fn normalize_xterm_sgr_compat_input(data: &str) -> String {
    let mut normalized = String::with_capacity(data.len());
    let mut index = 0;

    while let Some(relative_start) = data[index..].find("\x1b[") {
        let start = index + relative_start;
        normalized.push_str(&data[index..start]);

        let body_start = start + 2;
        let Some(relative_end) = data[body_start..].find('m') else {
            normalized.push_str(&data[start..]);
            return normalized;
        };
        let end = body_start + relative_end;
        let body = &data[body_start..end];

        if body.contains(':') {
            let rebuilt = body
                .split(';')
                .map(|token| {
                    normalize_xterm_truecolor_token(token).unwrap_or_else(|| token.to_string())
                })
                .collect::<Vec<_>>()
                .join(";");
            normalized.push_str("\x1b[");
            normalized.push_str(&rebuilt);
            normalized.push('m');
        } else {
            normalized.push_str(&data[start..=end]);
        }

        index = end + 1;
    }

    normalized.push_str(&data[index..]);
    normalized
}

pub fn serialize_terminal(
    terminal: &Terminal<'_, '_>,
    fixture_name: Option<&str>,
) -> Result<SerializeOutput, Box<dyn std::error::Error>> {
    let active_screen = terminal.active_screen()?;
    let mut serialized_candidate = String::new();
    let active = serialize_screen(terminal, active_screen)?;

    if active_screen == ffi::TerminalScreen::ALTERNATE {
        if let Some(primary) = serialize_screen(terminal, ffi::TerminalScreen::PRIMARY)? {
            serialized_candidate.push_str(&primary);
        }
        if let Some(active) = active.as_deref() {
            let (_, remainder) = split_leading_sgr_prefix(active);
            if !remainder.is_empty() {
                let active_cursor = screen_cursor_meta(terminal, ffi::TerminalScreen::ALTERNATE)?
                    .ok_or(GhosttyError::InvalidValue)?;
                if let Some(prefix) =
                    style_prefix(active_cursor.cursor_style, &terminal.color_palette()?)
                {
                    serialized_candidate.push_str(&prefix);
                }
            }
        }
        serialized_candidate.push_str("\x1b[?1049h\x1b[H");
    }

    if let Some(active) = active {
        serialized_candidate.push_str(&active);
    }
    if terminal.mode(Mode::ORIGIN)? {
        serialized_candidate.push_str("\x1b[?6h");
    }
    if terminal.mode(Mode::DECCKM)? {
        serialized_candidate.push_str("\x1b[?1h");
    }
    if !terminal.mode(Mode::WRAPAROUND)? {
        serialized_candidate.push_str("\x1b[?7l");
    }
    if terminal.mode(Mode::REVERSE_WRAP)? {
        serialized_candidate.push_str("\x1b[?45h");
    }
    if terminal.mode(Mode::KEYPAD_KEYS)? {
        serialized_candidate.push_str("\x1b[?66h");
    }
    if terminal.mode(Mode::ANY_MOUSE)? {
        // xterm orders the mouse-tracking suffix after bracketed paste and
        // focus-reporting suffixes, so defer emission until after those.
    } else if terminal.mode(Mode::BUTTON_MOUSE)? {
        // see note above
    } else if terminal.mode(Mode::NORMAL_MOUSE)? {
        // see note above
    }
    if terminal.mode(Mode::LEFT_RIGHT_MARGIN)? {
        serialized_candidate.push_str("\x1b[?69h");
    }
    serialized_candidate.push_str(&scrolling_region_suffix(terminal)?);
    if terminal.mode(Mode::INSERT)? {
        serialized_candidate.push_str("\x1b[4h");
    }
    if terminal.mode(Mode::BRACKETED_PASTE)? {
        serialized_candidate.push_str("\x1b[?2004h");
    }
    if terminal.mode(Mode::FOCUS_EVENT)? {
        serialized_candidate.push_str("\x1b[?1004h");
    }
    if terminal.mode(Mode::ANY_MOUSE)? {
        serialized_candidate.push_str("\x1b[?1003h");
    } else if terminal.mode(Mode::BUTTON_MOUSE)? {
        serialized_candidate.push_str("\x1b[?1002h");
    } else if terminal.mode(Mode::NORMAL_MOUSE)? {
        serialized_candidate.push_str("\x1b[?1000h");
    }
    if !terminal.is_cursor_visible()? {
        serialized_candidate.push_str("\x1b[?25l");
    }

    Ok(SerializeOutput {
        fixture_name: fixture_name.map(ToOwned::to_owned),
        serialized_candidate,
        cursor_x: terminal.cursor_x()?,
        cursor_y: terminal.cursor_y()?,
    })
}

pub fn run_fixture_by_name(
    fixture_name: &str,
) -> Result<SerializeOutput, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(fixture_path(&fixture_name))?;
    let fixture: FixtureFile = serde_json::from_str(&raw)?;
    let mut terminal = Terminal::new(TerminalOptions {
        cols: 80,
        rows: 24,
        max_scrollback: 1000,
    })?;

    for chunk in &fixture.chunks {
        let normalized = normalize_xterm_sgr_compat_input(&chunk.data);
        terminal.vt_write(normalized.as_bytes());
    }

    serialize_terminal(&terminal, Some(&fixture.name))
}
