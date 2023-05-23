use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vt100::Screen;

/// Draw the [`Screen`] to the [`Buffer`],
/// area is the designated area that the consumer provides
pub fn handle_screen(screen: &Screen, area: &Rect, buf: &mut Buffer) {
    let cols = area.width;
    let rows = area.height;

    // The [`Screen`] is made out of rows of cells
    for row in 0..rows {
        for col in 0..cols {
            if let Some(screen_cell) = screen.cell(row, col) {
                if screen_cell.has_contents() {
                    let cell = buf.get_mut(col, row);
                    cell.set_symbol(&screen_cell.contents());
                }
            }
        }
    }
}
