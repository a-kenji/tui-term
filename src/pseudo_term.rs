use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termwiz::cell::Intensity;
use termwiz::escape::csi::{Cursor, DecPrivateMode, Edit, Mode, Sgr};
use termwiz::escape::{Action, ControlCode, Esc, OperatingSystemCommand, CSI};
use termwiz::surface::CursorShape;

pub mod state;
pub mod termwiz_action;

#[derive(Default, Debug)]
pub struct PseudoTermState {
    area: Rect,
    buf_area: Rect,

    fg: Option<termwiz::color::ColorSpec>,
    bg: Option<termwiz::color::ColorSpec>,
    cursor_style: Option<CursorShape>,
    dec_private_mode: Option<DecPrivateMode>,
    title: Option<String>,
    cursor: (u16, u16),

    dec_auto_wrap: bool,
}

impl PseudoTermState {
    // The default entrypoint
    fn handle_actions(&mut self, actions: &Vec<Action>, area: Rect, buf: &mut Buffer) {
        // println!("buf area: {:?}, area: {area:?}", buf.area);
        // println!(
        //     "area right: {:?}, area left: {:?}",
        //     area.right(),
        //     area.left()
        // );

        self.area = area;
        self.buf_area = buf.area;

        for action in actions {
            // println!(
            //     "action: {:?}, row: {}, col: {}",
            //     action,
            //     self.row(),
            //     self.col()
            // );
            match action {
                Action::Print(char) => {
                    buf.get_mut(self.col(), self.row()).set_char(*char);
                    self.advance_col();
                }
                Action::PrintString(string) => {
                    for char in string.chars() {
                        buf.get_mut(self.col(), self.row()).set_char(char);
                        self.advance_col();
                    }
                }
                Action::Control(control) => {
                    self.handle_control(control, area, buf);
                }
                Action::DeviceControl(_) => todo!(),
                Action::OperatingSystemCommand(operating_system_command) => {
                    self.handle_operating_system_command(operating_system_command, area, buf)
                }
                Action::CSI(csi) => self.handle_csi(csi, area, buf),
                Action::Esc(esc) => self.handle_esc(esc, area, buf),
                Action::Sixel(_) => todo!(),
                Action::XtGetTcap(_) => todo!(),
                Action::KittyImage(_) => todo!(),
            }
        }
    }
    fn handle_csi(&mut self, csi: &CSI, _area: Rect, _buf: &mut Buffer) {
        match csi {
            CSI::Sgr(sgr) => self.handle_csi_sgr(sgr, _area, _buf),
            CSI::Cursor(cursor) => self.handle_csi_cursor(cursor, _area, _buf),
            CSI::Edit(edit) => self.handle_csi_edit(edit, _area, _buf),
            CSI::Mode(mode) => self.handle_csi_mode(mode, _area, _buf),
            CSI::Device(_) => todo!(),
            CSI::Mouse(_) => todo!(),
            CSI::Window(_) => todo!(),
            CSI::Keyboard(_) => todo!(),
            CSI::SelectCharacterPath(_, _) => todo!(),
            CSI::Unspecified(_) => todo!(),
        }
    }
    fn handle_esc(&mut self, esc: &Esc, _area: Rect, _buf: &mut Buffer) {
        match esc {
            Esc::Unspecified {
                intermediate,
                control,
            } => todo!(),
            Esc::Code(code) => match code {
                termwiz::escape::EscCode::FullReset => todo!(),
                termwiz::escape::EscCode::Index => todo!(),
                termwiz::escape::EscCode::NextLine => todo!(),
                termwiz::escape::EscCode::CursorPositionLowerLeft => todo!(),
                termwiz::escape::EscCode::HorizontalTabSet => {
                    // TODO: implement
                }
                termwiz::escape::EscCode::ReverseIndex => todo!(),
                termwiz::escape::EscCode::SingleShiftG2 => todo!(),
                termwiz::escape::EscCode::SingleShiftG3 => todo!(),
                termwiz::escape::EscCode::StartOfGuardedArea => todo!(),
                termwiz::escape::EscCode::EndOfGuardedArea => todo!(),
                termwiz::escape::EscCode::StartOfString => todo!(),
                termwiz::escape::EscCode::ReturnTerminalId => todo!(),
                termwiz::escape::EscCode::StringTerminator => todo!(),
                termwiz::escape::EscCode::PrivacyMessage => todo!(),
                termwiz::escape::EscCode::ApplicationProgramCommand => todo!(),
                termwiz::escape::EscCode::TmuxTitle => todo!(),
                termwiz::escape::EscCode::DecBackIndex => todo!(),
                termwiz::escape::EscCode::DecSaveCursorPosition => todo!(),
                termwiz::escape::EscCode::DecRestoreCursorPosition => todo!(),
                termwiz::escape::EscCode::DecApplicationKeyPad => todo!(),
                termwiz::escape::EscCode::DecNormalKeyPad => todo!(),
                termwiz::escape::EscCode::DecLineDrawingG0 => todo!(),
                termwiz::escape::EscCode::UkCharacterSetG0 => todo!(),
                termwiz::escape::EscCode::AsciiCharacterSetG0 => todo!(),
                termwiz::escape::EscCode::DecLineDrawingG1 => todo!(),
                termwiz::escape::EscCode::UkCharacterSetG1 => todo!(),
                termwiz::escape::EscCode::AsciiCharacterSetG1 => todo!(),
                termwiz::escape::EscCode::DecScreenAlignmentDisplay => todo!(),
                termwiz::escape::EscCode::DecDoubleHeightTopHalfLine => todo!(),
                termwiz::escape::EscCode::DecDoubleHeightBottomHalfLine => todo!(),
                termwiz::escape::EscCode::DecSingleWidthLine => todo!(),
                termwiz::escape::EscCode::DecDoubleWidthLine => todo!(),
                termwiz::escape::EscCode::ApplicationModeArrowUpPress => todo!(),
                termwiz::escape::EscCode::ApplicationModeArrowDownPress => todo!(),
                termwiz::escape::EscCode::ApplicationModeArrowRightPress => todo!(),
                termwiz::escape::EscCode::ApplicationModeArrowLeftPress => todo!(),
                termwiz::escape::EscCode::ApplicationModeHomePress => todo!(),
                termwiz::escape::EscCode::ApplicationModeEndPress => todo!(),
                termwiz::escape::EscCode::F1Press => todo!(),
                termwiz::escape::EscCode::F2Press => todo!(),
                termwiz::escape::EscCode::F3Press => todo!(),
                termwiz::escape::EscCode::F4Press => todo!(),
            },
        }
    }
    fn handle_csi_mode(&mut self, mode: &Mode, _area: Rect, _buf: &mut Buffer) {
        match mode {
            Mode::SetDecPrivateMode(mode) => {
                self.handle_csi_mode_set_deq_private_mode(mode, _area, _buf)
            }
            Mode::ResetDecPrivateMode(_) => {
                // TODO: implement
            }
            Mode::SaveDecPrivateMode(_) => todo!(),
            Mode::RestoreDecPrivateMode(_) => todo!(),
            Mode::QueryDecPrivateMode(_) => todo!(),
            Mode::SetMode(_) => todo!(),
            Mode::ResetMode(_) => todo!(),
            Mode::QueryMode(_) => todo!(),
            Mode::XtermKeyMode { resource, value } => todo!(),
        }
    }
    fn handle_csi_sgr(&mut self, sgr: &Sgr, _area: Rect, _buf: &mut Buffer) {
        match sgr {
            Sgr::Reset => self.sgr_reset(),
            Sgr::Intensity(intensity) => self.handle_csi_sgr_intensity(intensity, _area, _buf),
            Sgr::Underline(_) => todo!(),
            Sgr::UnderlineColor(_) => todo!(),
            Sgr::Blink(_) => todo!(),
            Sgr::Italic(_) => todo!(),
            Sgr::Inverse(_) => todo!(),
            Sgr::Invisible(_) => todo!(),
            Sgr::StrikeThrough(_) => todo!(),
            Sgr::Font(_) => todo!(),
            Sgr::Foreground(palette) => {
                self.fg = Some(*palette);
            }
            Sgr::Background(_) => todo!(),
            Sgr::Overline(_) => todo!(),
            Sgr::VerticalAlign(_) => todo!(),
        }
    }
    fn handle_csi_sgr_intensity(&mut self, intensity: &Intensity, _area: Rect, _buf: &mut Buffer) {
        match intensity {
            Intensity::Normal => {
                // TODO: implement
            }
            Intensity::Bold => {
                // TODO: implement
            }
            Intensity::Half => {
                // TODO: implement
            }
        }
    }
    fn handle_csi_edit(&mut self, edit: &Edit, _area: Rect, _buf: &mut Buffer) {
        match edit {
            Edit::DeleteCharacter(_) => todo!(),
            Edit::DeleteLine(_) => todo!(),
            Edit::EraseCharacter(_) => todo!(),
            Edit::EraseInLine(_) => {
                // TODO: implement
            }
            Edit::InsertCharacter(_) => todo!(),
            Edit::InsertLine(_) => todo!(),
            Edit::ScrollDown(_) => todo!(),
            Edit::ScrollUp(_) => todo!(),
            Edit::EraseInDisplay(_) => {
                // TODO: implement
            }
            Edit::Repeat(_) => todo!(),
        }
    }
    fn handle_csi_cursor(&mut self, cursor: &Cursor, _area: Rect, _buf: &mut Buffer) {
        match cursor {
            Cursor::BackwardTabulation(_) => todo!(),
            Cursor::TabulationClear(tabulation_clear) => match tabulation_clear {
                termwiz::escape::csi::TabulationClear::ClearCharacterTabStopAtActivePosition => {
                    // TODO: implement
                }
                termwiz::escape::csi::TabulationClear::ClearLineTabStopAtActiveLine => {
                    // TODO: implement
                }
                termwiz::escape::csi::TabulationClear::ClearCharacterTabStopsAtActiveLine => {
                    // TODO: implement
                }
                termwiz::escape::csi::TabulationClear::ClearAllCharacterTabStops => {
                    // TODO: implement
                }
                termwiz::escape::csi::TabulationClear::ClearAllLineTabStops => todo!(),
                termwiz::escape::csi::TabulationClear::ClearAllTabStops => todo!(),
            },
            Cursor::CharacterAbsolute(_) => todo!(),
            Cursor::CharacterPositionAbsolute(_) => todo!(),
            Cursor::CharacterPositionBackward(_) => todo!(),
            Cursor::CharacterPositionForward(_) => todo!(),
            Cursor::CharacterAndLinePosition { line, col } => todo!(),
            Cursor::LinePositionAbsolute(_) => todo!(),
            Cursor::LinePositionBackward(_) => todo!(),
            Cursor::LinePositionForward(_) => todo!(),
            Cursor::ForwardTabulation(_) => todo!(),
            Cursor::NextLine(_) => todo!(),
            Cursor::PrecedingLine(_) => todo!(),
            Cursor::ActivePositionReport { line, col } => todo!(),
            Cursor::RequestActivePositionReport => todo!(),
            Cursor::SaveCursor => todo!(),
            Cursor::RestoreCursor => todo!(),
            Cursor::TabulationControl(_) => todo!(),
            Cursor::Left(amount) => {
                let cols = self.col();
                self.set_col(cols - *amount as u16);
            }
            Cursor::Down(amount) => {
                let rows = self.row();
                self.set_row(rows + *amount as u16);
            }
            Cursor::Right(amount) => {
                let cols = self.col();
                self.set_col(cols + *amount as u16);
            }
            Cursor::Position { line, col } => {
                self.set_col(col.as_zero_based() as u16);
                self.set_row(line.as_zero_based() as u16);
            }
            Cursor::Up(amount) => {
                let rows = self.row();
                self.set_row(rows - *amount as u16);
            }
            Cursor::LineTabulation(_) => todo!(),
            Cursor::SetTopAndBottomMargins { top, bottom } => todo!(),
            Cursor::SetLeftAndRightMargins { left, right } => todo!(),
            Cursor::CursorStyle(cursor_style) => {}
        }
    }
    fn handle_control(&mut self, control_code: &ControlCode, _area: Rect, _buf: &mut Buffer) {
        match control_code {
            ControlCode::Null => todo!(),
            ControlCode::StartOfHeading => todo!(),
            ControlCode::StartOfText => todo!(),
            ControlCode::EndOfText => todo!(),
            ControlCode::EndOfTransmission => todo!(),
            ControlCode::Enquiry => todo!(),
            ControlCode::Acknowledge => todo!(),
            ControlCode::Bell => todo!(),
            ControlCode::Backspace => todo!(),
            ControlCode::HorizontalTab => {
                // TODO: implement
                // Move cursor to next tab stop
                // Need to save tab stops
            }
            ControlCode::LineFeed => {
                let row = self.row();
                self.set_row(row + 1);
            }
            ControlCode::VerticalTab => todo!(),
            ControlCode::FormFeed => todo!(),
            ControlCode::CarriageReturn => {
                self.set_col(0);
            }
            ControlCode::ShiftOut => {
                // ignored
            }
            ControlCode::ShiftIn => {
                // ignored
            }
            ControlCode::DataLinkEscape => todo!(),
            ControlCode::DeviceControlOne => todo!(),
            ControlCode::DeviceControlTwo => todo!(),
            ControlCode::DeviceControlThree => todo!(),
            ControlCode::DeviceControlFour => todo!(),
            ControlCode::NegativeAcknowledge => todo!(),
            ControlCode::SynchronousIdle => todo!(),
            ControlCode::EndOfTransmissionBlock => todo!(),
            ControlCode::Cancel => todo!(),
            ControlCode::EndOfMedium => todo!(),
            ControlCode::Substitute => todo!(),
            ControlCode::Escape => todo!(),
            ControlCode::FileSeparator => todo!(),
            ControlCode::GroupSeparator => todo!(),
            ControlCode::RecordSeparator => todo!(),
            ControlCode::UnitSeparator => todo!(),
            ControlCode::BPH => todo!(),
            ControlCode::NBH => todo!(),
            ControlCode::IND => todo!(),
            ControlCode::NEL => todo!(),
            ControlCode::SSA => todo!(),
            ControlCode::ESA => todo!(),
            ControlCode::HTS => todo!(),
            ControlCode::HTJ => todo!(),
            ControlCode::VTS => todo!(),
            ControlCode::PLD => todo!(),
            ControlCode::PLU => todo!(),
            ControlCode::RI => todo!(),
            ControlCode::SS2 => todo!(),
            ControlCode::SS3 => todo!(),
            ControlCode::DCS => todo!(),
            ControlCode::PU1 => todo!(),
            ControlCode::PU2 => todo!(),
            ControlCode::STS => todo!(),
            ControlCode::CCH => todo!(),
            ControlCode::MW => todo!(),
            ControlCode::SPA => todo!(),
            ControlCode::EPA => todo!(),
            ControlCode::SOS => todo!(),
            ControlCode::SCI => todo!(),
            ControlCode::CSI => todo!(),
            ControlCode::ST => todo!(),
            ControlCode::OSC => todo!(),
            ControlCode::PM => todo!(),
            ControlCode::APC => todo!(),
        }
    }
    fn handle_csi_mode_set_deq_private_mode(
        &mut self,
        mode: &DecPrivateMode,
        _area: Rect,
        _buf: &mut Buffer,
    ) {
        match mode {
            DecPrivateMode::Code(dec_private_mode) => {
                match dec_private_mode {
                    termwiz::escape::csi::DecPrivateModeCode::ApplicationCursorKeys => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::DecAnsiMode => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::Select132Columns => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SmoothScroll => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::ReverseVideo => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::OriginMode => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::AutoWrap => {
                        self.dec_auto_wrap = true;
                    }
                    termwiz::escape::csi::DecPrivateModeCode::AutoRepeat => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::StartBlinkingCursor => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::ShowCursor => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::ReverseWraparound => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::LeftRightMarginMode => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SixelDisplayMode => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::MouseTracking => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::HighlightMouseTracking => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::ButtonEventMouse => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::AnyEventMouse => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::FocusTracking => {
                        // TODO: this should be handled by the terminal
                    }
                    termwiz::escape::csi::DecPrivateModeCode::Utf8Mouse => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SGRMouse => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SGRPixelsMouse => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::XTermMetaSendsEscape => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::XTermAltSendsEscape => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SaveCursor => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::ClearAndEnableAlternateScreen => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::EnableAlternateScreen => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::OptEnableAlternateScreen => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::BracketedPaste => {
                        // TODO: this should be handled by the terminal
                    }
                    termwiz::escape::csi::DecPrivateModeCode::UsePrivateColorRegistersForEachGraphic => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SynchronizedOutput => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::MinTTYApplicationEscapeKeyMode => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::SixelScrollsRight => todo!(),
                    termwiz::escape::csi::DecPrivateModeCode::Win32InputMode => todo!(),
                }
            }
            DecPrivateMode::Unspecified(_) => todo!(),
        }
    }
    fn handle_operating_system_command(
        &mut self,
        operating_system_command: &OperatingSystemCommand,
        _area: Rect,
        _buf: &mut Buffer,
    ) {
        match operating_system_command {
            OperatingSystemCommand::SetIconNameAndWindowTitle(title) => {
                // TODO: set icon name
                self.title = Some(title.into());
            }
            OperatingSystemCommand::SetWindowTitle(_) => todo!(),
            OperatingSystemCommand::SetWindowTitleSun(_) => todo!(),
            OperatingSystemCommand::SetIconName(_) => todo!(),
            OperatingSystemCommand::SetIconNameSun(_) => todo!(),
            OperatingSystemCommand::SetHyperlink(_) => todo!(),
            OperatingSystemCommand::ClearSelection(_) => todo!(),
            OperatingSystemCommand::QuerySelection(_) => todo!(),
            OperatingSystemCommand::SetSelection(_, _) => todo!(),
            OperatingSystemCommand::SystemNotification(_) => todo!(),
            OperatingSystemCommand::ITermProprietary(_) => todo!(),
            OperatingSystemCommand::FinalTermSemanticPrompt(_) => todo!(),
            OperatingSystemCommand::ChangeColorNumber(_) => todo!(),
            OperatingSystemCommand::ChangeDynamicColors(_, _) => todo!(),
            OperatingSystemCommand::ResetDynamicColor(_) => todo!(),
            OperatingSystemCommand::CurrentWorkingDirectory(_) => todo!(),
            OperatingSystemCommand::ResetColors(_) => todo!(),
            OperatingSystemCommand::RxvtExtension(_) => todo!(),
            OperatingSystemCommand::Unspecified(_) => todo!(),
        }
    }

    /// Sets sgr attributes to their default values.
    fn sgr_reset(&mut self) {
        self.fg = None;
        self.bg = None;
    }
    fn row(&self) -> u16 {
        self.cursor.0
    }
    fn col(&self) -> u16 {
        self.cursor.1
    }
    fn row_mut(&mut self) -> u16 {
        self.cursor.0
    }
    fn col_mut(&mut self) -> u16 {
        self.cursor.1
    }
    fn set_col(&mut self, col: u16) {
        self.cursor.1 = col;
    }
    fn set_row(&mut self, row: u16) {
        self.cursor.0 = row;
    }
    fn advance_col(&mut self) {
        if self.dec_auto_wrap && self.col() >= self.area.right() - 1 {
            self.advance_row();
            self.cursor.1 = 0;
        } else {
            self.cursor.1 += 1;
        }
    }
    fn advance_row(&mut self) {
        if self.row() >= self.area.bottom() - 1 {
            // println!("Buttom reached");
        } else {
            self.cursor.0 += 1;
        }
    }
}
