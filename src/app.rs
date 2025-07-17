// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{self, Frame, Geometry, Path};
use cosmic::iced::{Alignment, Color, Length, Point, Rectangle, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, button, dialog, icon, menu, nav_bar};
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
    Tick(Instant),
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
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
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
            Page::Page1 => widget::column()
                .push(widget::text::title1(fl!("kawaii-title")))
                .push(widget::text(fl!("kawaii-welcome")))
                .push(
                    widget::row()
                        .push(widget::text("🐱"))
                        .push(widget::text("💖"))
                        .push(widget::text("🎀"))
                        .push(widget::text("🌙"))
                        .push(widget::text("⭐"))
                        .spacing(10),
                )
                .push(widget::text(fl!("kawaii-face")))
                .push(widget::button::standard(fl!("kawaii-button")).on_press(Message::TogglePopup))
                .push(widget::text(fl!("kawaii-footer")))
                .spacing(20)
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
            Page::Page2 => widget::column()
                .push(widget::text::title1("Page 2 Content"))
                .push(widget::text("This is page 2 with custom content!"))
                .push(widget::button::standard("Click me").on_press(Message::SubscriptionChannel))
                .spacing(20)
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
            Page::Page3 => widget::text::title1("Page 3")
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
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
            cosmic::iced::time::every(Duration::from_millis(16)).map(Message::Tick),
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

            Message::Tick(instant) => {
                self.animation_time = instant;
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
#[derive(Copy, Clone)]
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
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

impl canvas::Program<Message> for KawaiiCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &cosmic::iced::Renderer,
        _theme: &cosmic::iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let time = self.animation_time.elapsed().as_secs_f32();

        // Kawaii background gradient circles
        for i in 0..5 {
            let angle = time * 0.5 + i as f32 * 1.2;
            let radius = 30.0 + (time * 2.0 + i as f32).sin() * 10.0;
            let x = center.x + angle.cos() * (50.0 + i as f32 * 20.0);
            let y = center.y + angle.sin() * (30.0 + i as f32 * 15.0);

            let circle = Path::circle(Point::new(x, y), radius);
            let color = match i % 4 {
                0 => Color::from_rgba(1.0, 0.7, 0.8, 0.3), // Pink
                1 => Color::from_rgba(0.8, 0.9, 1.0, 0.3), // Light blue
                2 => Color::from_rgba(1.0, 1.0, 0.8, 0.3), // Light yellow
                _ => Color::from_rgba(0.9, 0.8, 1.0, 0.3), // Light purple
            };
            frame.fill(&circle, color);
        }

        // Floating hearts (simplified as circles)
        for i in 0..8 {
            let t = time * 1.5 + i as f32 * 0.8;
            let x = center.x + (t * 0.7).cos() * (80.0 + i as f32 * 15.0);
            let y = center.y + (t * 0.5).sin() * (60.0 + i as f32 * 10.0) - (t * 20.0).sin() * 5.0;

            // Draw simple heart shape using circles
            let heart_size = 6.0 + (t * 3.0).sin() * 2.0;
            let heart = Path::circle(Point::new(x, y), heart_size);

            frame.fill(&heart, Color::from_rgba(1.0, 0.4, 0.6, 0.8));
        }

        // Sparkle stars
        for i in 0..12 {
            let t = time * 2.0 + i as f32 * 0.5;
            let x = center.x + (t * 1.2).cos() * (100.0 + i as f32 * 12.0);
            let y = center.y + (t * 0.8).sin() * (80.0 + i as f32 * 8.0);
            let size = 3.0 + (t * 4.0).sin().abs() * 2.0;

            // 4-pointed star
            let star = Path::new(|path| {
                path.move_to(Point::new(x, y - size));
                path.line_to(Point::new(x + size * 0.3, y - size * 0.3));
                path.line_to(Point::new(x + size, y));
                path.line_to(Point::new(x + size * 0.3, y + size * 0.3));
                path.line_to(Point::new(x, y + size));
                path.line_to(Point::new(x - size * 0.3, y + size * 0.3));
                path.line_to(Point::new(x - size, y));
                path.line_to(Point::new(x - size * 0.3, y - size * 0.3));
                path.close();
            });

            frame.fill(&star, Color::from_rgba(1.0, 1.0, 0.6, 0.9));
        }

        vec![frame.into_geometry()]
    }
}
