use sfml::{ graphics::*, system::*, window::* };

use seamonkey::*;
use core::CoreAction;
use context::{ Context, Action };
use input::*;
use editor::Editor;

pub struct PoetWindow<'w> {
    size: Vector2f,
    window: RenderWindow,
    surface: RectangleShape<'w>,
    framebuffer: RenderTexture,
    editor: Editor,
    focused: bool,
}

impl<'w> PoetWindow<'w> {

    pub fn editor() -> Status<Self> {

        let size = Vector2f::new(400.0, 400.0);

        let mut window = RenderWindow::new((400, 400), "poet", Style::DEFAULT, &Default::default());
        window.set_vertical_sync_enabled(true);

        let mut surface = RectangleShape::with_size(size);
        let framebuffer = RenderTexture::new(400, 400, false).unwrap();

        let texture_pointer = framebuffer.texture() as *const _;
        surface.set_texture(unsafe { &*texture_pointer }, false);

        let mut editor = confirm!(Editor::new()); // remove mut
        confirm!(editor.open_file(format_shared!("/home/maybe")));

        return success!(Self {
            size: size,
            window: window,
            surface: surface,
            framebuffer: framebuffer,
            editor: editor,
            focused: true,
        });
    }

    pub fn handle_input(&mut self, context: &Context) -> Option<CoreAction> {

        'handle: while let Some(event) = self.window.poll_event() {
            match event {

                Event::Closed => return Some(CoreAction::CloseWindow),

                Event::KeyPressed { code, shift, ctrl, alt, system } => {
                    if !is_modifier_key(code) {
                        let modifiers = Modifiers::from(shift, ctrl, alt, system);

                        println!("modifiers: {:?}", modifiers);

                        let key_event = KeyEvent::new(code, modifiers);

                        println!("key event: {:?}", key_event);

                        for action in context.get_matching_actions(&key_event) {

                            println!("action: {:?}", action);

                            if display!(self.editor.handle_action(context, action)) {
                                self.rerender(context);
                                continue 'handle;
                            }

                            // put actions into a queue instead?
                            match action {
                                Action::NewEditor => return Some(CoreAction::NewEditor),
                                Action::ZoomIn => return Some(CoreAction::ZoomIn),
                                Action::ZoomOut => return Some(CoreAction::ZoomOut),
                                Action::Quit => return Some(CoreAction::Quit),
                                Action::CloseWindow => self.close(),
                                unhandeled => {},
                            }
                        }
                    }
                    //println!("code: {:?} shift: {} ctrl: {} alt: {} system: {}", code, shift, ctrl, alt, system);
                    //return Some(CoreAction::NewEditor);
                },

                Event::Resized { width, height } => {
                    self.resize(context, Vector2f::new(width as f32, height as f32));
                    self.rerender(context);
                },

                Event::GainedFocus => {
                    self.focused = true;
                    self.rerender(context);
                },

                Event::LostFocus => {
                    self.focused = false;
                    self.rerender(context);
                },

                ignored => {},
            }
        }

        return None;
    }

    pub fn resize(&mut self, context: &Context, size: Vector2f) {

        self.size = size;

        let view = View::from_rect(&FloatRect::new(0.0, 0.0, size.x as f32, size.y as f32));
        self.window.set_view(&view);

        self.surface = RectangleShape::with_size(size);
        self.framebuffer = RenderTexture::new(size.x as u32, size.y as u32, false).unwrap();

        let texture_pointer = self.framebuffer.texture() as *const _;
        self.surface.set_texture(unsafe { &*texture_pointer }, false);

        self.editor.resize(context, size);
    }

    pub fn rerender(&mut self, context: &Context) {
        self.editor.rerender(&mut self.framebuffer, context, self.focused);
    }

    pub fn display(&mut self) {
        self.window.clear(Color::BLACK);
        self.window.draw(&self.surface);
        self.window.display();
    }

    pub fn close(&mut self) {
        self.window.close();
    }
}
