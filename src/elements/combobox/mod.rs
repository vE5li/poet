mod theme;

use seamonkey::*;

use sfml::graphics::*;
use sfml::system::Vector2f;

use dialogues::DialogueTheme;
use elements::{ TextBox, Textfield };
use interface::{ InterfaceContext, InterfaceTheme };
use system::{ LanguageManager, subtract_or_zero };
use input::Action;

pub use self::theme::ElementTheme;

macro_rules! handle_return_none {
    ($expression: expr) => ({
        $expression;
        return (true, None);
    })
}

macro_rules! handle_maybe_return_none {
    ($expression: expr) => ({
        if ($expression) {
            return (true, None);
        }
    })
}

#[derive(Clone)]
pub enum ComboSelection {
    TextBox,
    Variant(usize, SharedString),
}

impl ComboSelection {

    pub fn is_textbox(&self) -> bool {
        match self {
            ComboSelection::TextBox => return true,
            _other => return false,
        }
    }

    pub fn index_matches(&self, selected: usize) -> bool {
        match self {
            ComboSelection::TextBox => return false,
            ComboSelection::Variant(index, _original) => return *index == selected,
        }
    }
}

pub struct ComboBox {
    pub textbox: TextBox,
    pub allow_unknown: bool,
    pub variants: Vec<SharedString>,
    pub selection: ComboSelection,
    pub displacement: usize,
    pub path_mode: bool,
    pub scroll: usize,
    pub size: Vector2f,
    pub position: Vector2f,
    line_count: usize,
}

impl ComboBox {

    pub fn new(language_manager: &mut LanguageManager, description: &'static str, displacement: usize, allow_unknown: bool, path_mode: bool, variants: Vec<SharedString>) -> Self {
        Self {
            textbox: TextBox::new(language_manager, description, displacement),
            allow_unknown: allow_unknown,
            variants: variants,
            selection: ComboSelection::TextBox,
            displacement: displacement,
            path_mode: path_mode,
            scroll: 0,
            size: Vector2f::new(0., 0.),
            position: Vector2f::new(0., 0.),
            line_count: 0,
        }
    }

    fn move_up(&mut self, interface_context: &InterfaceContext, language_manager: &mut LanguageManager) {
        if let ComboSelection::Variant(index, original) = self.selection.clone() {
            if index == 0 {
                self.selection = ComboSelection::TextBox;
                self.textbox.set_text(language_manager, original);
            } else {
                let new_index = index - 1;
                let valid_variants = self.valid_variants();
                self.selection = ComboSelection::Variant(new_index, original.clone());

                match self.path_mode {
                    true => self.textbox.set_text_without_save(language_manager, self.get_combined(&valid_variants[new_index])),
                    false => self.textbox.set_text_without_save(language_manager, valid_variants[new_index].clone())
                }

                self.check_selection_gaps(interface_context, new_index);
            }
        }
    }

    fn move_down(&mut self, interface_context: &InterfaceContext, language_manager: &mut LanguageManager) {
        if let ComboSelection::TextBox = self.selection.clone() {
            let valid_variants = self.valid_variants();
            if !valid_variants.is_empty() {
                self.selection = ComboSelection::Variant(0, self.textbox.get());

                match self.path_mode {
                    true => self.textbox.set_text_without_save(language_manager, self.get_combined(&valid_variants[0])),
                    false => self.textbox.set_text_without_save(language_manager, valid_variants[0].clone())
                }
            }
            return;
        }

        if let ComboSelection::Variant(index, original) = self.selection.clone() {
            let valid_variants = self.valid_variants();

            if index + 1 < valid_variants.len() {
                self.selection = ComboSelection::Variant(index + 1, original.clone());
                self.textbox.set_text(language_manager, valid_variants[index + 1].clone());

                match self.path_mode {
                    true => self.textbox.set_text_without_save(language_manager, self.get_combined(&valid_variants[index + 1])),
                    false => self.textbox.set_text_without_save(language_manager, valid_variants[index + 1].clone())
                }

                self.check_selection_gaps(interface_context, index + 1);
            }
        }
    }

    fn check_selection_gaps(&mut self, interface_context: &InterfaceContext, index: usize) {
        if interface_context.selection_gap * 2 >= self.line_count {
            self.scroll = subtract_or_zero(index, self.line_count / 2);
        } else if interface_context.selection_gap + self.scroll > index {
            self.scroll = subtract_or_zero(index, interface_context.selection_gap);
        } else if index + 1 - self.scroll + interface_context.selection_gap > self.line_count {
            self.scroll = index + 1 - (self.line_count - interface_context.selection_gap);
        }
    }

    pub fn get_combined(&self, suffix: &SharedString) -> SharedString {
        let original = match &self.selection {
            ComboSelection::Variant(_index, original) => original.clone(),
            ComboSelection::TextBox => self.textbox.get(),
        };

        let positions = original.position(&SharedString::from("/"));
        if !positions.is_empty() {
            let mut combined = original.slice(0, positions[positions.len() - 1]);
            combined.push_str(suffix);
            return combined;
        }

        return suffix.clone();
    }

    pub fn valid_variants(&self) -> Vec<SharedString> {
        let mut original = match &self.selection {
            ComboSelection::Variant(_index, original) => original.clone(),
            ComboSelection::TextBox => self.textbox.get(),
        };

        if self.path_mode {
            let pieces = original.split(&SharedString::from("/"), false);
            original = pieces[pieces.len() - 1].clone();
        }

        let mut valid_variants = self.variants.clone();
        valid_variants.retain(|variant| variant.contains(&original));
        return valid_variants;
    }

    pub fn remove_selected_variant(&mut self, interface_context: &InterfaceContext, language_manager: &mut LanguageManager) {
        if let ComboSelection::Variant(index, original) = &self.selection {
            self.variants.remove(*index);
        }

        if let ComboSelection::Variant(index, original) = self.selection.clone() {
            let valid_variants = self.valid_variants();

            if valid_variants.is_empty() {
                self.selection = ComboSelection::TextBox;
                self.textbox.set_text(language_manager, original);
                return;
            }

            let new_index = match index >= valid_variants.len() {
                true => index - 1,
                false => index,
            };

            self.selection = ComboSelection::Variant(new_index, original.clone());

            match self.path_mode {
                true => self.textbox.set_text_without_save(language_manager, self.get_combined(&valid_variants[new_index])),
                false => self.textbox.set_text_without_save(language_manager, valid_variants[new_index].clone())
            }

            self.check_selection_gaps(interface_context, new_index);
        }
    }

    pub fn get(&self) -> SharedString {
        return self.textbox.get();
    }

    pub fn is_textbox_focused(&self) -> bool {
        return self.selection.is_textbox();
    }

    pub fn set_text(&mut self, language_manager: &mut LanguageManager, text: SharedString) {
        self.textbox.set_text(language_manager, text);
    }

    pub fn clear(&mut self, language_manager: &mut LanguageManager) {
        self.reset_selection();
        self.textbox.clear(language_manager);
    }

    pub fn reset_selection(&mut self) {
        self.selection = ComboSelection::TextBox;
        self.scroll = 0;
    }

    fn focus_next(&mut self, language_manager: &mut LanguageManager) -> bool {
        let valid_variants = self.valid_variants();

        if valid_variants.is_empty() {
            return false;
        }

        let suffix = match &self.selection {
            ComboSelection::Variant(index, _original) => valid_variants[*index].clone(),
            ComboSelection::TextBox => valid_variants[0].clone(),
        };

        match self.path_mode {
            true => self.textbox.set_text(language_manager, self.get_combined(&suffix)),
            false => self.textbox.set_text(language_manager, suffix),
        }

        self.reset_selection();
        return true;
    }

    fn handle_confirm(&mut self, language_manager: &mut LanguageManager) -> bool {

        if !self.allow_unknown && self.selection.is_textbox() {
            let valid_variants = self.valid_variants();
            if valid_variants.is_empty() {
                return true;
            }

            match self.path_mode {
                true => self.textbox.set_text(language_manager, self.get_combined(&valid_variants[0])),
                false => self.textbox.set_text(language_manager, valid_variants[0].clone()),
            }

            return self.path_mode && *self.textbox.get().chars().last().unwrap() == Character::from_char('/');
        }

        return false;
    }

    pub fn handle_action(&mut self, interface_context: &InterfaceContext, language_manager: &mut LanguageManager, action: Action) -> (bool, Option<bool>) {
        match action {

            Action::Up => handle_return_none!(self.move_up(interface_context, language_manager)),

            Action::Down => handle_return_none!(self.move_down(interface_context, language_manager)),

            Action::FocusNext => handle_maybe_return_none!(self.focus_next(language_manager)),

            Action::Left => self.reset_selection(),

            Action::Right => self.reset_selection(),

            Action::Start => self.reset_selection(),

            Action::End => self.reset_selection(),

            Action::Remove => self.reset_selection(),

            Action::Delete => self.reset_selection(),

            Action::DeleteLine => self.reset_selection(),

            Action::ExtendLeft => self.reset_selection(),

            Action::ExtendRight => self.reset_selection(),

            Action::MoveLeft => self.reset_selection(),

            Action::MoveRight => self.reset_selection(),

            Action::Copy => self.reset_selection(),

            Action::Paste => self.reset_selection(),

            Action::Cut => self.reset_selection(),

            Action::Undo => self.reset_selection(),

            Action::Redo => self.reset_selection(),

            _other => { },
        }

        if let Some(action) = self.textbox.handle_action(language_manager, action) {
            match action {

                Action::Confirm => {
                    match self.handle_confirm(language_manager) {
                        true => return (true, None),
                        false => return (true, Some(true)),
                    }
                }

                Action::Abort => return (true, Some(false)),

                _unhandled => return (false, None),
            }
        }

        return (true, None);
    }

    pub fn add_character(&mut self, language_manager: &mut LanguageManager, character: Character) {
        self.reset_selection();
        self.textbox.add_character(language_manager, character);
    }

    pub fn update_layout(&mut self, interface_context: &InterfaceContext, theme: &DialogueTheme, size: Vector2f, position: Vector2f) {
        self.textbox.update_layout(interface_context, theme, size, position);

        let float_font_size = interface_context.font_size as f32;
        let height = (size.y - theme.height * float_font_size) * theme.display_height;
        let element_height = (theme.height * float_font_size) + (theme.unfocused_element_theme.padding * float_font_size);

        self.line_count = (height / element_height) as usize;
        self.size = size;
        self.position = position;

        if let ComboSelection::Variant(index, ..) = self.selection.clone() {
            self.check_selection_gaps(interface_context, index);
        }
    }

    pub fn render(&self, framebuffer: &mut RenderTexture, interface_context: &InterfaceContext, theme: &DialogueTheme, focused: bool) {
        self.textbox.render(framebuffer, interface_context, theme, focused && self.selection.is_textbox());

        if focused {

            let padding = match self.selection.is_textbox() {
                true => theme.focused_textbox_theme.padding * interface_context.font_size as f32,
                false => theme.unfocused_textbox_theme.padding * interface_context.font_size as f32,
            };

            let dialogue_height = theme.height * interface_context.font_size as f32;
            let mut top_position = self.position.y + padding + (self.displacement + 1) as f32 * dialogue_height;
            let size = Vector2f::new(self.size.x, dialogue_height);
            let valid_variants = self.valid_variants();

            for index in self.scroll..valid_variants.len() {
                if top_position > self.size.y || index - self.scroll >= self.line_count {
                    break;
                }

                let element_theme = match self.selection.index_matches(index) {
                    true => &theme.focused_element_theme,
                    false => &theme.unfocused_element_theme,
                };

                let position = Vector2f::new(self.position.x, top_position);
                Textfield::render(framebuffer, interface_context, &element_theme.textfield_theme, &valid_variants[index], size, position, dialogue_height);
                top_position += dialogue_height + element_theme.padding * interface_context.font_size as f32;
            }
        }
    }
}
