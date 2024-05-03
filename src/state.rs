use ratatui::{buffer::Buffer, layout::Rect};

use crate::widget::{Cell, PseudoTerminal, Screen};

/// Draw the [`Screen`] to the [`Buffer`],
/// area is the designated area that the consumer provides
pub fn handle<S: Screen>(term: &PseudoTerminal<S>, area: Rect, buf: &mut Buffer) {
    let cols = area.width;
    let rows = area.height;
    let col_start = area.x;
    let row_start = area.y;
    let area_cols = area.width + area.x;
    let area_rows = area.height + area.y;
    let screen = term.screen();

    // The [`Screen`] is made out of rows of cells
    for row in 0..rows {
        for col in 0..cols {
            let buf_col = col + col_start;
            let buf_row = row + row_start;

            if buf_row > area_rows || buf_col > area_cols {
                // Skip writing outside the area
                continue;
            }

            if let Some(screen_cell) = screen.cell(row, col) {
                let cell = buf.get_mut(buf_col, buf_row);
                screen_cell.apply(cell);
            }
        }
    }

    if !screen.hide_cursor() && term.cursor.show {
        let (c_row, c_col) = screen.cursor_position();
        if (c_row + row_start) < area_rows && (c_col + col_start) < area_cols {
            let c_cell = buf.get_mut(c_col + col_start, c_row + row_start);
            if let Some(cell) = screen.cell(c_row, c_col) {
                if cell.has_contents() {
                    let style = term.cursor.overlay_style;
                    c_cell.set_style(style);
                } else {
                    let symbol = &term.cursor.symbol;
                    let style = term.cursor.style;
                    c_cell.set_symbol(symbol);
                    c_cell.set_style(style);
                }
            }
        }
    }
}
