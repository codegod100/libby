// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::mouse;
use cosmic::widget::canvas::{self, Frame, Geometry, Path};
use cosmic::iced::{Alignment, Color, Length, Point, Rectangle, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, button, dialog, icon, menu, nav_bar};
use cosmic::iced::widget::Stack;
use cosmic::{cosmic_theme, theme};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,
    /// Animation state for kawaii canvas
    animation_time: Instant,
    show_popup: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    OpenAuthorUrl,
    SubscriptionChannel,
    ToggleContextPage(ContextPage),
    TogglePopup,
    UpdateConfig(Config),
    LaunchUrl(String),
    Tick,
    GoToPage3,
    UpdateUsername(String),
    SaveSettings,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.codegod100.libby";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar with three page items.
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("page-id", num = 1))
            .data::<Page>(Page::Page1)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text(fl!("page-id", num = 2))
            .data::<Page>(Page::Page2)
            .icon(icon::from_name("applications-system-symbolic"));

        nav.insert()
            .text(fl!("page-id", num = 3))
            .data::<Page>(Page::Page3)
            .icon(icon::from_name("applications-games-symbolic"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            animation_time: Instant::now(),
            show_popup: false,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        (app, command)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                    menu::Item::Button("Settings".to_string(), None, MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title("Settings"),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let active_page = self
            .nav
            .data::<Page>(self.nav.active())
            .copied()
            .unwrap_or(Page::Page1);

        match active_page {
            Page::Page1 => {
                let canvas = cosmic::widget::canvas(KawaiiCanvas::new(self.animation_time))
                    .width(Length::Fill)
                    .height(Length::Fill);

                let text_content = widget::column()
                    .push(widget::text::title1("Welcome to the Kawaii Canvas!"))
                    .push(widget::text("Move your mouse around to see the shapes react."))
                    .push(widget::button::standard("Click me").on_press(Message::TogglePopup))
                    .spacing(10)
                    .padding(20)
                    .align_x(Horizontal::Center)
                    .width(Length::Fill);

                let stack = Stack::new()
                    .push(canvas)
                    .push(
                        widget::container(text_content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center),
                    );

                stack.into()
            },
            Page::Page2 => widget::column()
                .push(widget::text::title1("Page 2 Content"))
                .push(widget::text("This is page 2 with custom content!"))
                .push(widget::button::standard("Click me").on_press(Message::GoToPage3))
                .spacing(20)
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
            Page::Page3 => {
                let display_username = if self.config.username.is_empty() {
                    // Fallback to OS username
                    std::env::var("USER")
                        .or_else(|_| std::env::var("USERNAME"))
                        .unwrap_or_else(|_| "Unknown User".to_string())
                } else {
                    self.config.username.clone()
                };
                
                let username_text = widget::text::title2(format!("Hello, {}!", display_username));
                let info_text = if self.config.username.is_empty() {
                    widget::text("Using OS username. Go to Settings in the View menu to set a custom username.")
                } else {
                    widget::text("Go to Settings in the View menu to update your username")
                };
                
                widget::column()
                    .push(widget::text::title1("Page 3"))
                    .push(widget::vertical_space().height(20))
                    .push(username_text)
                    .push(widget::vertical_space().height(10))
                    .push(info_text)
                    .spacing(10)
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into()
            }
        }
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Animation timer for kawaii canvas
            cosmic::iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }
            Message::OpenAuthorUrl => {
                _ = open::that_detached(
                    "https://deer.social/profile/did:plc:ngokl2gnmpbvuvrfckja3g7p",
                );
            }

            Message::SubscriptionChannel => {
                println!("button clicked");
                // For example purposes only.
            }

            Message::TogglePopup => {
                self.show_popup = !self.show_popup;
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::Tick => {}

            Message::GoToPage3 => {
                // Find the nav ID for page 3
                let page3_id = self.nav.iter().find(|&id| {
                    self.nav.data::<Page>(id).copied() == Some(Page::Page3)
                });
                
                if let Some(id) = page3_id {
                    self.nav.activate(id);
                    return self.update_title();
                }
            }

            Message::UpdateUsername(username) => {
                self.config.username = username;
            }

            Message::SaveSettings => {
                // Save config to persistent storage
                if let Ok(config_context) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                    let _ = self.config.write_entry(&config_context);
                }
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }

    fn dialog(&self) -> Option<Element<Message>> {
        if self.show_popup {
            let active_page = self
                .nav
                .data::<Page>(self.nav.active())
                .copied()
                .unwrap_or(Page::Page1);

            match active_page {
                Page::Page1 => Some(
                    dialog()
                        .title("This is a popup on page 1!")
                        .body("This is the body of the popup.")
                        .icon(icon::from_name("face-cool-symbolic"))
                        .primary_action(
                            button::standard("Close").on_press(Message::TogglePopup)
                        )
                        .into(),
                ),
                _ => None,
            }
        } else {
            None
        }
    }

    
}

impl AppModel {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title2(fl!("app-title"));
        let author = widget::button::link("nandi.weird.one").on_press(Message::OpenAuthorUrl);

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(author)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// The settings page for this app.
    pub fn settings(&self) -> Element<Message> {
        widget::column()
            .push(widget::text::title2("Settings"))
            .push(widget::vertical_space().height(20))
            .push(widget::text("Username:"))
            .push(
                widget::text_input("Enter your username", &self.config.username)
                    .on_input(Message::UpdateUsername)
                    .width(Length::Fill)
            )
            .push(widget::vertical_space().height(20))
            .push(
                widget::button::standard("Save Settings")
                    .on_press(Message::SaveSettings)
                    .width(Length::Fill)
            )
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

/// The page to display in the application.
#[derive(Copy, Clone, PartialEq)]
pub enum Page {
    Page1,
    Page2,
    Page3,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}

/// Kawaii animated canvas with floating hearts and sparkles
pub struct KawaiiCanvas {
    animation_time: Instant,
}

impl KawaiiCanvas {
    pub fn new(animation_time: Instant) -> Self {
        Self { animation_time }
    }
}

impl canvas::Program<Message, cosmic::Theme, cosmic::Renderer> for KawaiiCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::Renderer,
        _theme: &cosmic::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let time = self.animation_time.elapsed().as_secs_f32();
        
        // Use modulo for smooth looping - 30 second loop
        let loop_duration = 30.0;
        let loop_time = (time % loop_duration) * (std::f32::consts::PI * 2.0) / loop_duration;

        // Mouse avoidance parameters
        let mouse_pos = if let Some(pos) = cursor.position() {
            Point::new(pos.x - bounds.x, pos.y - bounds.y)
        } else {
            Point::new(-1.0, -1.0)
        };
        let avoidance_radius = 20.0;
        let repulsion_strength = 15.0;

        // Kawaii background gradient circles with smooth loops
        for i in 0..5 {
            let phase = i as f32 * 1.2566; // 2π/5 for even distribution
            let angle = loop_time * 0.3 + phase;
            let radius = 30.0 + (loop_time * 1.5 + phase).sin() * 8.0;
            let orbit_radius = 60.0 + i as f32 * 25.0;
            let mut x = center.x + angle.cos() * orbit_radius;
            let mut y = center.y + angle.sin() * orbit_radius * 0.7; // Slightly elliptical

            // Mouse avoidance
            let dx = x - mouse_pos.x;
            let dy = y - mouse_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < avoidance_radius {
                let repel_factor = (1.0 - distance / avoidance_radius) * repulsion_strength;
                x += dx / distance * repel_factor;
                y += dy / distance * repel_factor;
            }

            let circle = Path::circle(Point::new(x, y), radius);
            let color = match i % 4 {
                0 => Color::from_rgba(1.0, 0.7, 0.8, 0.4), // Pink
                1 => Color::from_rgba(0.8, 0.9, 1.0, 0.4), // Light blue
                2 => Color::from_rgba(1.0, 1.0, 0.8, 0.4), // Light yellow
                _ => Color::from_rgba(0.9, 0.8, 1.0, 0.4), // Light purple
            };
            frame.fill(&circle, color);
        }

        // Floating hearts with smooth circular motion
        for i in 0..8 {
            let phase = i as f32 * 0.785; // 2π/8 for even distribution
            let t = loop_time * 0.8 + phase;
            let orbit_radius = 90.0 + (i % 3) as f32 * 20.0;
            let mut x = center.x + t.cos() * orbit_radius;
            let mut y = center.y + t.sin() * orbit_radius * 0.6 + (t * 2.0).sin() * 15.0;

            // Mouse avoidance
            let dx = x - mouse_pos.x;
            let dy = y - mouse_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < avoidance_radius {
                let repel_factor = (1.0 - distance / avoidance_radius) * repulsion_strength;
                x += dx / distance * repel_factor;
                y += dy / distance * repel_factor;
            }

                        // Pulsing heart size
            let heart_size = 8.0 + (t * 2.5).sin() * 3.0;
            let heart = Path::new(|path| {
                path.move_to(Point::new(x, y + heart_size * 0.25));
                path.bezier_curve_to(
                    Point::new(x + heart_size * 0.5, y - heart_size * 0.5),
                    Point::new(x + heart_size, y),
                    Point::new(x, y + heart_size),
                );
                path.bezier_curve_to(
                    Point::new(x - heart_size, y),
                    Point::new(x - heart_size * 0.5, y - heart_size * 0.5),
                    Point::new(x, y + heart_size * 0.25),
                );
                path.close();
            });

            frame.fill(&heart, Color::from_rgba(1.0, 0.4, 0.6, 0.7));
        }

        // Sparkle stars with smooth rotation
        for i in 0..12 {
            let phase = i as f32 * 0.524; // 2π/12 for even distribution
            let t = loop_time * 1.2 + phase;
            let orbit_radius = 120.0 + (i % 4) as f32 * 15.0;
            let mut x = center.x + t.cos() * orbit_radius;
            let mut y = center.y + t.sin() * orbit_radius * 0.8;
            let size = 4.0 + (t * 3.0).sin().abs() * 2.0;

            // Mouse avoidance
            let dx = x - mouse_pos.x;
            let dy = y - mouse_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < avoidance_radius {
                let repel_factor = (1.0 - distance / avoidance_radius) * repulsion_strength;
                x += dx / distance * repel_factor;
                y += dy / distance * repel_factor;
            }

            // 4-pointed star with smooth rotation
            let star_rotation = t * 0.5;
            let star = Path::new(|path| {
                let cos_r = star_rotation.cos();
                let sin_r = star_rotation.sin();
                
                // Rotate the star points
                let points = [
                    (0.0, -size),
                    (size * 0.3, -size * 0.3),
                    (size, 0.0),
                    (size * 0.3, size * 0.3),
                    (0.0, size),
                    (-size * 0.3, size * 0.3),
                    (-size, 0.0),
                    (-size * 0.3, -size * 0.3),
                ];
                
                let first_point = points[0];
                let rotated_x = first_point.0 * cos_r - first_point.1 * sin_r;
                let rotated_y = first_point.0 * sin_r + first_point.1 * cos_r;
                path.move_to(Point::new(x + rotated_x, y + rotated_y));
                
                for &point in &points[1..] {
                    let rot_x = point.0 * cos_r - point.1 * sin_r;
                    let rot_y = point.0 * sin_r + point.1 * cos_r;
                    path.line_to(Point::new(x + rot_x, y + rot_y));
                }
                path.close();
            });

            frame.fill(&star, Color::from_rgba(1.0, 1.0, 0.6, 0.8));
        }

        vec![frame.into_geometry()]
    }
}
