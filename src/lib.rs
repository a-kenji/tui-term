use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use termwiz::escape::Action;

pub struct PseudoTerm<'a> {
    actions: &'a Vec<Action>,
}

impl<'a> PseudoTerm<'a> {
    pub fn new(actions: &'a Vec<Action>) -> Self {
        PseudoTerm { actions }
    }
}

impl Widget for PseudoTerm<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for action in self.actions {
            match action {
                Action::Print(_) => todo!(),
                Action::PrintString(_) => todo!(),
                Action::Control(_) => todo!(),
                Action::DeviceControl(_) => todo!(),
                Action::OperatingSystemCommand(_) => todo!(),
                Action::CSI(_) => todo!(),
                Action::Esc(_) => todo!(),
                Action::Sixel(_) => todo!(),
                Action::XtGetTcap(_) => todo!(),
                Action::KittyImage(_) => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn simple_ls_output() {
        let actions = vec![]
    }
}
