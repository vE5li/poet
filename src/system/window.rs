use sfml::{ graphics::*, system::*, window::* };

use seamonkey::*;
use input::*;
use interface::{ Interface, InterfaceTheme, InterfaceContext };
use system::{ ResourceManager, LanguageManager };
use elements::TextbufferContext;
use input::Action;

pub struct PoetWindow<'w> {
    size: Vector2f,
    window: RenderWindow,
    surface: RectangleShape<'w>,
    framebuffer: RenderTexture,
    interface: Interface,
    focused: bool,
}

impl<'w> PoetWindow<'w> {

    pub fn interface(interface_context: &InterfaceContext, resource_manager: &mut ResourceManager, language_manager: &mut LanguageManager, window_id: usize) -> Status<Self> {

        let size = Vector2f::new(400.0, 400.0);

        let mut window = RenderWindow::new((400, 400), "poet", Style::DEFAULT, &Default::default());
        window.set_vertical_sync_enabled(true);

        let mut settings = ContextSettings::default();
        settings.set_antialiasing_level(interface_context.antialiasing_level as u32);
        let framebuffer = RenderTexture::with_settings(400, 400, &settings).unwrap();
        let mut surface = RectangleShape::with_size(size);

        let texture_pointer = framebuffer.texture() as *const _;
        surface.set_texture(unsafe { &*texture_pointer }, false);

        let interface = confirm!(Interface::new(resource_manager, language_manager, window_id));

        return success!(Self {
            size: size,
            window: window,
            surface: surface,
            framebuffer: framebuffer,
            interface: interface,
            focused: true,
        });
    }

    pub fn handle_input(&mut self, interface_context: &InterfaceContext, textbuffer_context: &TextbufferContext, theme: &InterfaceTheme, resource_manager: &mut ResourceManager, language_manager: &mut LanguageManager, theme_name: &mut SharedString) -> Vec<Action> {
        let mut action_queue = Vec::new();
        let mut force_rerender = false;
        let mut handled = false;

        if self.interface.history_catch_up(textbuffer_context, resource_manager) {
            force_rerender = true;
        }

        'handle: while let Some(event) = self.window.poll_event() {
            match event {

                Event::Closed => action_queue.push(Action::CloseWindow),

                Event::KeyPressed { code, shift, ctrl, alt, system } => {
                    if !is_modifier_key(code) {
                        let modifiers = Modifiers::from(shift, ctrl, alt, system);
                        let key_event = KeyEvent::new(code, modifiers);

                        for action in interface_context.get_matching_actions(&key_event) {
                            if let Some(unhandled_action) = self.interface.handle_action(interface_context, textbuffer_context, resource_manager, language_manager, action, theme_name) {
                                if unhandled_action.is_global() {
                                    action_queue.push(unhandled_action);
                                    handled = true;
                                    continue 'handle;
                                }
                            } else {
                                self.rerender(interface_context, textbuffer_context, theme, resource_manager);
                                handled = true;
                                continue 'handle;
                            }
                        }
                    }
                },

                Event::TextEntered { unicode } => {
                    if handled {
                        handled = false;
                        continue 'handle;
                    }

                    let character = match unicode as usize {
                        13 => Character::from_char('\n'),
                        0...31 => continue 'handle,
                        32...126 => Character::from_char(unicode),
                        _other => continue 'handle,
                    };

                    self.interface.add_character(textbuffer_context, resource_manager, language_manager, character);
                    force_rerender = true;
                },

                Event::Resized { width, height } => {
                    self.size = Vector2f::new(width as f32, height as f32);
                    self.reallocate(interface_context);
                    self.update_layout(interface_context, textbuffer_context, resource_manager, theme);
                    force_rerender = true;
                },

                Event::GainedFocus => {
                    self.focused = true;
                    force_rerender = true;
                },

                Event::LostFocus => {
                    self.focused = false;
                    force_rerender = true;
                },

                Event::MouseWheelScrolled { delta, .. } => {
                    match delta > 0.0 {
                        true => self.interface.scroll_up(textbuffer_context),
                        false => self.interface.scroll_down(textbuffer_context),
                    }
                    force_rerender = true;
                },

                _ignored => {},
            }
        }

        if force_rerender {
            self.rerender(interface_context, textbuffer_context, theme, resource_manager);
        }

        return action_queue;
    }

    pub fn reallocate(&mut self, interface_context: &InterfaceContext) {
        let view = View::from_rect(&FloatRect::new(0.0, 0.0, self.size.x as f32, self.size.y as f32));
        self.window.set_view(&view);

        let mut settings = ContextSettings::default();
        settings.set_antialiasing_level(interface_context.antialiasing_level as u32);
        self.framebuffer = RenderTexture::with_settings(self.size.x as u32, self.size.y as u32, &settings).unwrap();
        self.surface = RectangleShape::with_size(self.size);

        let texture_pointer = self.framebuffer.texture() as *const _;
        self.surface.set_texture(unsafe { &*texture_pointer }, false);
    }

    pub fn update_layout(&mut self, interface_context: &InterfaceContext, textbuffer_context: &TextbufferContext, resource_manager: &ResourceManager, theme: &InterfaceTheme) {
        self.interface.update_layout(interface_context, textbuffer_context, resource_manager, theme, self.size);
    }

    pub fn rerender(&mut self, interface_context: &InterfaceContext, textbuffer_context: &TextbufferContext, theme: &InterfaceTheme, resource_manager: &ResourceManager) {
        self.framebuffer.clear(Color::BLACK);
        self.interface.render(&mut self.framebuffer, interface_context, textbuffer_context, theme, resource_manager, self.focused);
        self.framebuffer.display();
    }

    pub fn display(&mut self) {
        self.window.clear(Color::BLACK);
        self.window.draw(&self.surface);
        self.window.display();
    }

    pub fn set_error_state(&mut self, error: Error) {
        self.interface.set_error_state(error);
    }

    pub fn close(&mut self) {
        self.window.close();
    }
}
