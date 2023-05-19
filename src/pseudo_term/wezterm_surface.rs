use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use termwiz::escape::Action;
use termwiz::surface::Surface;

pub struct PseudoTerm<'a> {
    surface: &'a Surface,
}

impl<'a> PseudoTerm<'a> {
    pub fn new(surface: &'a Surface) -> Self {
        PseudoTerm { surface }
    }
}

impl Widget for PseudoTerm<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let mut row = area.height;
        // let mut col = area.width;
        let mut row = 0;
        let mut col = 0;

        let surface = self.surface;

        if surface.has_changes(0) {
            panic!("Surface has changes!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_ls_output() {
        let actions = vec![];
    }
}
