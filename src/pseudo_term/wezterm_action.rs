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
        // let mut row = area.height;
        // let mut col = area.width;
        let mut row = area.height;
        let mut col = area.width;

        for action in self.actions {
            match action {
                Action::Print(char) => {
                    buf.get_mut(col, row).set_char(*char);
                    col += 1;
                }
                Action::PrintString(string) => {
                    for char in string.chars() {
                        buf.get_mut(col, row).set_char(char);
                    }
                }
                Action::Control(control) => match control {
                    termwiz::escape::ControlCode::Null => todo!(),
                    termwiz::escape::ControlCode::StartOfHeading => todo!(),
                    termwiz::escape::ControlCode::StartOfText => todo!(),
                    termwiz::escape::ControlCode::EndOfText => todo!(),
                    termwiz::escape::ControlCode::EndOfTransmission => todo!(),
                    termwiz::escape::ControlCode::Enquiry => todo!(),
                    termwiz::escape::ControlCode::Acknowledge => todo!(),
                    termwiz::escape::ControlCode::Bell => todo!(),
                    termwiz::escape::ControlCode::Backspace => todo!(),
                    termwiz::escape::ControlCode::HorizontalTab => {
                        // Move to next tab character
                    }
                    termwiz::escape::ControlCode::LineFeed => {
                        row += 1;
                    }
                    termwiz::escape::ControlCode::VerticalTab => todo!(),
                    termwiz::escape::ControlCode::FormFeed => todo!(),
                    termwiz::escape::ControlCode::CarriageReturn => {
                        col = 0;
                    }
                    termwiz::escape::ControlCode::ShiftOut => todo!(),
                    termwiz::escape::ControlCode::ShiftIn => todo!(),
                    termwiz::escape::ControlCode::DataLinkEscape => todo!(),
                    termwiz::escape::ControlCode::DeviceControlOne => todo!(),
                    termwiz::escape::ControlCode::DeviceControlTwo => todo!(),
                    termwiz::escape::ControlCode::DeviceControlThree => todo!(),
                    termwiz::escape::ControlCode::DeviceControlFour => todo!(),
                    termwiz::escape::ControlCode::NegativeAcknowledge => todo!(),
                    termwiz::escape::ControlCode::SynchronousIdle => todo!(),
                    termwiz::escape::ControlCode::EndOfTransmissionBlock => todo!(),
                    termwiz::escape::ControlCode::Cancel => todo!(),
                    termwiz::escape::ControlCode::EndOfMedium => todo!(),
                    termwiz::escape::ControlCode::Substitute => todo!(),
                    termwiz::escape::ControlCode::Escape => todo!(),
                    termwiz::escape::ControlCode::FileSeparator => todo!(),
                    termwiz::escape::ControlCode::GroupSeparator => todo!(),
                    termwiz::escape::ControlCode::RecordSeparator => todo!(),
                    termwiz::escape::ControlCode::UnitSeparator => todo!(),
                    termwiz::escape::ControlCode::BPH => todo!(),
                    termwiz::escape::ControlCode::NBH => todo!(),
                    termwiz::escape::ControlCode::IND => todo!(),
                    termwiz::escape::ControlCode::NEL => todo!(),
                    termwiz::escape::ControlCode::SSA => todo!(),
                    termwiz::escape::ControlCode::ESA => todo!(),
                    termwiz::escape::ControlCode::HTS => todo!(),
                    termwiz::escape::ControlCode::HTJ => todo!(),
                    termwiz::escape::ControlCode::VTS => todo!(),
                    termwiz::escape::ControlCode::PLD => todo!(),
                    termwiz::escape::ControlCode::PLU => todo!(),
                    termwiz::escape::ControlCode::RI => todo!(),
                    termwiz::escape::ControlCode::SS2 => todo!(),
                    termwiz::escape::ControlCode::SS3 => todo!(),
                    termwiz::escape::ControlCode::DCS => todo!(),
                    termwiz::escape::ControlCode::PU1 => todo!(),
                    termwiz::escape::ControlCode::PU2 => todo!(),
                    termwiz::escape::ControlCode::STS => todo!(),
                    termwiz::escape::ControlCode::CCH => todo!(),
                    termwiz::escape::ControlCode::MW => todo!(),
                    termwiz::escape::ControlCode::SPA => todo!(),
                    termwiz::escape::ControlCode::EPA => todo!(),
                    termwiz::escape::ControlCode::SOS => todo!(),
                    termwiz::escape::ControlCode::SCI => todo!(),
                    termwiz::escape::ControlCode::CSI => todo!(),
                    termwiz::escape::ControlCode::ST => todo!(),
                    termwiz::escape::ControlCode::OSC => todo!(),
                    termwiz::escape::ControlCode::PM => todo!(),
                    termwiz::escape::ControlCode::APC => todo!(),
                },
                Action::DeviceControl(_) => {}
                Action::OperatingSystemCommand(_) => {}
                Action::CSI(_) => {}
                Action::Esc(_) => {}
                Action::Sixel(_) => {}
                Action::XtGetTcap(_) => {}
                Action::KittyImage(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;

    #[test]
    fn empty_actions() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let actions = vec![];
        let pseudo_term = PseudoTerm::new(&actions);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
        insta::assert_snapshot!(view);
    }

    #[test]
    fn simple_ls() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let actions = vec![];
        let pseudo_term = PseudoTerm::new(&actions);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
        insta::assert_snapshot!(view);
    }
}
