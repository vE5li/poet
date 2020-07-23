use kami::*;
use context::{ Context, Action };
use sfml::graphics::*;
use sfml::system::Vector2f;
use super::super::ComboBox;

pub struct SetLanguageDialogue {
    pub language_box: ComboBox,
}

impl SetLanguageDialogue {

    pub fn new() -> Self {
        Self {
            language_box: ComboBox::new("language name", 0, false, false, vec![VectorString::from("cipher"), VectorString::from("c++"), VectorString::from("default"), VectorString::from("doofenshmirtz"), VectorString::from("entleman"), VectorString::from("none"), VectorString::from("rust"), VectorString::from("seamonkey")]),
        }
    }

    pub fn handle_action(&mut self, context: &Context, action: Action) -> (bool, Option<bool>) {

        if let Action::SetLanguage = action {
            return (true, Some(false));
        }

        return self.language_box.handle_action(context, action);
    }

    pub fn add_character(&mut self, character: Character) {
        self.language_box.add_character(character);
    }

    pub fn draw(&self, framebuffer: &mut RenderTexture, context: &Context, size: Vector2f, offset: Vector2f) {
        self.language_box.draw(framebuffer, context, size, offset, true);
    }
}
