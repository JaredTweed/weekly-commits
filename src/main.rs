use cosmic::app::Core;
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id as SurfaceId;
use cosmic::iced::{Alignment, Background, Border, Length, Limits, Subscription};
use cosmic::{Element, Task, widget};
use cosmic_weekly_commits::weekly;
use std::time::Duration;

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    PopupClosed(SurfaceId),
    Refresh,
    Loaded(weekly::Contributions),
    OpenSettings,
    Tick,
}

struct Applet {
    core: Core,
    popup: Option<SurfaceId>,
    settings: weekly::Settings,
    data: weekly::Contributions,
    refreshing: bool,
}

impl cosmic::Application for Applet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = weekly::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let settings = weekly::load_settings();
        let data = weekly::Contributions {
            dates: weekly::dates_for(&settings),
            counts: [0; 7],
            cached_at: None,
        };
        let app = Self {
            core,
            popup: None,
            settings: settings.clone(),
            data,
            refreshing: true,
        };
        (
            app,
            Task::perform(weekly::fetch_with_cache(settings), |data| {
                cosmic::Action::App(Message::Loaded(data))
            }),
        )
    }

    fn on_close_requested(&self, id: SurfaceId) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                if let Some(popup) = self.popup.take() {
                    return destroy_popup(popup);
                }
                let new_id = SurfaceId::unique();
                self.popup = Some(new_id);
                let mut settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    None,
                    None,
                    None,
                );
                settings.positioner.size_limits = Limits::NONE
                    .min_width(300.0)
                    .max_width(360.0)
                    .min_height(1.0)
                    .max_height(520.0);
                return get_popup(settings);
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Refresh | Message::Tick => {
                self.settings = weekly::load_settings();
                self.refreshing = true;
                let settings = self.settings.clone();
                return Task::perform(weekly::fetch_with_cache(settings), |data| {
                    cosmic::Action::App(Message::Loaded(data))
                });
            }
            Message::Loaded(data) => {
                self.data = data;
                self.refreshing = false;
            }
            Message::OpenSettings => {
                let _ = launch_settings();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let row = self.data.counts.iter().enumerate().fold(
            widget::Row::new().spacing(weekly::BOX_MARGIN),
            |row, (index, count)| {
                row.push(commit_box(
                    *count,
                    self.settings.theme_name,
                    self.settings.color_mode,
                    self.settings.highlight_current_day && self.data.dates[index] == today(),
                ))
            },
        );

        let button = self
            .core
            .applet
            .button_from_element(row.align_y(Alignment::Center), false)
            .on_press(Message::TogglePopup);

        Element::from(self.core.applet.applet_tooltip(
            button,
            "Weekly Commits",
            self.popup.is_some(),
            |_| Message::TogglePopup,
            None,
        ))
    }

    fn view_window(&self, id: SurfaceId) -> Element<'_, Self::Message> {
        if self.popup != Some(id) {
            return widget::text("").into();
        }

        let header = widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(widget::text::body(format!(
                "Commits - {}",
                self.settings.service_type.label()
            )))
            .push(widget::Space::new().width(Length::Fill));

        let table = self.data.dates.iter().zip(self.data.counts.iter()).fold(
            widget::Column::new().spacing(4),
            |column, (date, count)| {
                column.push(
                    widget::Row::new()
                        .spacing(12)
                        .align_y(Alignment::Center)
                        .push(widget::text::body(weekly::date_label(*date)).width(Length::Fill))
                        .push(
                            widget::text::body(count.to_string())
                                .width(Length::Fixed(48.0))
                                .align_x(cosmic::iced::alignment::Horizontal::Right),
                        ),
                )
            },
        );

        let status = if self.settings.github_username.trim().is_empty()
            || self.settings.github_token.trim().is_empty()
        {
            Some("No commit data available".to_string())
        } else if self.refreshing {
            Some("Refreshing...".to_string())
        } else {
            self.data
                .cached_at
                .as_ref()
                .map(|cached_at| weekly::cached_label(cached_at))
        };

        let mut content = widget::Column::new()
            .spacing(10)
            .padding(12)
            .push(header)
            .push(
                widget::Row::new()
                    .push(widget::text::caption("Date").width(Length::Fill))
                    .push(
                        widget::text::caption("Commits")
                            .width(Length::Fixed(58.0))
                            .align_x(cosmic::iced::alignment::Horizontal::Right),
                    ),
            )
            .push(table);

        if let Some(status) = status {
            content = content.push(widget::text::caption(status));
        }

        content = content
            .push(widget::divider::horizontal::default())
            .push(
                widget::button::standard("Refresh Now")
                    .on_press(Message::Refresh)
                    .width(Length::Fill),
            )
            .push(
                widget::button::standard("Settings")
                    .on_press(Message::OpenSettings)
                    .width(Length::Fill),
            );

        Element::from(self.core.applet.popup_container(content))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::time::every(Duration::from_secs(self.settings.refresh_interval.max(60)))
            .map(|_| Message::Tick)
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

fn commit_box<'a>(
    count: u32,
    theme_name: weekly::ThemeName,
    color_mode: weekly::ColorMode,
    highlight: bool,
) -> Element<'a, Message> {
    let color = weekly::hex_to_rgba(
        weekly::box_hex_color(count, theme_name, color_mode),
        if count == 0 {
            0.12
        } else {
            weekly::box_alpha(count, color_mode)
        },
    );
    let border_width = if highlight { 2.0 } else { 1.0 };
    let border_color = if highlight {
        cosmic::iced::Color::from_rgba8(255, 255, 255, 0.6)
    } else {
        cosmic::iced::Color::from_rgba8(255, 255, 255, 0.08)
    };

    widget::container(widget::Space::new().width(Length::Fixed(weekly::BOX_SIZE)))
        .width(Length::Fixed(weekly::BOX_SIZE))
        .height(Length::Fixed(weekly::BOX_SIZE))
        .style(move |_| widget::container::Style {
            text_color: None,
            background: Some(Background::Color(color)),
            border: Border {
                radius: weekly::BORDER_RADIUS.into(),
                width: border_width,
                color: border_color,
            },
            shadow: Default::default(),
            icon_color: None,
            snap: true,
        })
        .into()
}

fn today() -> time::Date {
    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .date()
}

fn launch_settings() -> std::io::Result<()> {
    let local_binary = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join("cosmic-weekly-commits-settings"))
        })
        .filter(|path| path.exists());

    let binary = local_binary.unwrap_or_else(|| "cosmic-weekly-commits-settings".into());
    std::process::Command::new(binary).spawn().map(|_| ())
}

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();
    cosmic::applet::run::<Applet>(())
}
